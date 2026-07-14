use crate::components::avatar::ImageAvatar;
use crate::components::button::Button;
use crate::components::card::*;
use crate::components::input::Input;
use crate::components::label::Label;
use crate::components::spinner::Spinner;
use crate::state::app_state::AppState;
use crate::state::notifications::{Notification, NotificationType, NotificationsState};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use dioxus::html::FileData;
use dioxus::prelude::*;
use matrix_sdk::media::{MediaFormat, MediaRequestParameters, MediaThumbnailSettings};
use matrix_sdk::ruma::events::room::MediaSource;
use matrix_sdk::ruma::media::Method;

#[css_module("src/routes/settings/components/profile.css")]
struct Styles;

#[component]
pub fn ProfileCard() -> Element {
    let state = use_context::<AppState>();
    let notifications = use_context::<NotificationsState>();

    let mut display_name = use_signal(String::new);
    let mut selected_avatar = use_signal(|| None::<FileData>);
    let mut selected_avatar_preview = use_signal(|| None::<String>);
    let is_saving = use_signal(|| false);

    // fetch current user details
    let mut profile_resource = use_resource(move || {
        let matrix = state.matrix.cloned();
        async move {
            let client = matrix.client().await?;
            let display_name = client.account().get_display_name().await.ok().flatten();
            let avatar_url = client.account().get_avatar_url().await.ok().flatten();

            let mut resolved_avatar = None;
            if let Some(ref mxc) = avatar_url {
                let format = MediaFormat::Thumbnail(MediaThumbnailSettings {
                    method: Method::Crop,
                    width: 100u32.into(),
                    height: 100u32.into(),
                    animated: false,
                });
                let request = MediaRequestParameters {
                    source: MediaSource::Plain(mxc.clone()),
                    format,
                };
                if let Ok(bytes) = client.media().get_media_content(&request, true).await {
                    let b64 = STANDARD.encode(bytes);
                    resolved_avatar = Some(format!("data:image/png;base64,{}", b64));
                }
            }
            Some((display_name, resolved_avatar))
        }
    });

    // Populate display name when loaded
    if let Some(Some((Some(name), _))) = &*profile_resource.read_unchecked()
        && display_name().is_empty()
    {
        display_name.set(name.clone());
    }

    let current_avatar_preview = match &*profile_resource.read_unchecked() {
        Some(Some((_, Some(preview)))) => Some(preview.clone()),
        _ => None,
    };

    let avatar_to_render = selected_avatar_preview().or(current_avatar_preview);

    let initials = display_name()
        .split_whitespace()
        .next()
        .and_then(|w| w.chars().next())
        .unwrap_or('?')
        .to_uppercase()
        .to_string();

    let onchange_avatar = move |e: FormEvent| {
        if let Some(file) = e.files().first().cloned() {
            selected_avatar.set(Some(file.clone()));
            spawn(async move {
                if let Ok(bytes) = file.read_bytes().await {
                    let b64 = STANDARD.encode(&bytes);
                    selected_avatar_preview.set(Some(format!("data:image/png;base64,{}", b64)));
                }
            });
        }
    };

    let onsubmit = move |e: FormEvent| {
        e.prevent_default();
        let matrix = state.matrix.cloned();
        let mut notifications = notifications;
        let mut is_saving = is_saving;
        let selected_file = selected_avatar.read().clone();
        let name_to_save = display_name.read().clone();

        spawn(async move {
            is_saving.set(true);
            let client = match matrix.client().await {
                Some(c) => c,
                None => {
                    notifications.push(Notification::new(
                        "Error",
                        "No Matrix client available",
                        NotificationType::Error,
                    ));
                    is_saving.set(false);
                    return;
                }
            };

            // update display name if set and not empty
            if !name_to_save.is_empty()
                && let Err(e) = client.account().set_display_name(Some(&name_to_save)).await
            {
                notifications.push(Notification::new(
                    "Error",
                    &format!("Failed to update display name: {}", e),
                    NotificationType::Error,
                ));
                is_saving.set(false);
                return;
            }

            // update avatar if new file is selected
            if let Some(file) = selected_file {
                let file_name = file.name();
                match file.read_bytes().await {
                    Ok(bytes) => {
                        let mime_type = mime_guess::from_path(std::path::Path::new(&file_name))
                            .first_or_octet_stream();

                        let parsed_mime: mime::Mime =
                            mime_type.to_string().parse().unwrap_or(mime::IMAGE_PNG);

                        match client
                            .media()
                            .upload(&parsed_mime, bytes.to_vec(), None)
                            .await
                        {
                            Ok(upload_res) => {
                                if let Err(e) = client
                                    .account()
                                    .set_avatar_url(Some(&upload_res.content_uri))
                                    .await
                                {
                                    notifications.push(Notification::new(
                                        "Error",
                                        &format!("Failed to set avatar URL: {}", e),
                                        NotificationType::Error,
                                    ));
                                    is_saving.set(false);
                                    return;
                                }
                            }
                            Err(e) => {
                                notifications.push(Notification::new(
                                    "Error",
                                    &format!("Failed to upload avatar: {}", e),
                                    NotificationType::Error,
                                ));
                                is_saving.set(false);
                                return;
                            }
                        }
                    }
                    Err(e) => {
                        notifications.push(Notification::new(
                            "Error",
                            &format!("Failed to read file bytes: {}", e),
                            NotificationType::Error,
                        ));
                        is_saving.set(false);
                        return;
                    }
                }
            }

            profile_resource.restart();
            notifications.push(Notification::new(
                "Success",
                "Profile updated successfully",
                NotificationType::Success,
            ));
            is_saving.set(false);
        });
    };

    rsx! {
        Card { class: Styles::settings_card,
            form { class: Styles::settings_form, onsubmit,

                div { class: Styles::avatar_edit_section,
                    label {
                        r#for: "avatar-file-input",
                        class: Styles::avatar_container,
                        ImageAvatar {
                            src: avatar_to_render.unwrap_or_default(),
                            "{initials}"
                        }
                        div { class: Styles::avatar_overlay, "Change" }
                    }
                    input {
                        r#type: "file",
                        id: "avatar-file-input",
                        class: Styles::hidden_file_input,
                        accept: "image/*",
                        onchange: onchange_avatar,
                    }
                }

                div { class: Styles::form_group,
                    Label { html_for: "display-name", "Display Name" }
                    Input {
                        id: "display-name",
                        class: Styles::input_field,
                        r#type: "text",
                        placeholder: "Enter display name",
                        value: "{display_name}",
                        oninput: move |e: Event<FormData>| display_name.set(e.value()),
                    }
                }

                Button {
                    r#type: "submit",
                    class: Styles::save_button,
                    disabled: *is_saving.read(),
                    if *is_saving.read() {
                        Spinner {}
                        span { "Saving..." }
                    } else {
                        span { "Save Profile" }
                    }
                }
            }
        }
    }
}
