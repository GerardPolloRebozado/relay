use crate::components::avatar::{AvatarImageSize, ImageAvatar};
use crate::components::button::Button;
use crate::components::card::*;
use crate::components::input::Input;
use crate::components::label::Label;
use crate::components::spinner::Spinner;
use crate::state::app_state::AppState;
use crate::state::notifications::{Notification, NotificationType, NotificationsState};
use crate::utilities::media::{AvatarSize, get_user_profile_avatar};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use dioxus::html::FileData;
use dioxus::prelude::*;

#[css_module("src/routes/settings/components/profile.css")]
struct Styles;

#[derive(Clone)]
struct UserProfile {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub matrix_id: String,
}

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
            let resolved_avatar = get_user_profile_avatar(&client, AvatarSize::Large).await;
            let matrix_id = client.user_id().unwrap().to_string();

            Some(UserProfile {
                display_name,
                avatar_url: resolved_avatar,
                matrix_id,
            })
        }
    });

    let Some(Some(profile)) = profile_resource.cloned() else {
        return rsx! {
            Card { class: Styles::settings_card,
                div { class: Styles::avatar_edit_section, Spinner {} }
            }
        };
    };

    // Populate display name when loaded
    if display_name.read().is_empty()
        && let Some(ref name) = profile.display_name
    {
        display_name.set(name.clone());
    }

    let avatar_to_render = selected_avatar_preview().or(profile.avatar_url.clone());

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
                            size: AvatarImageSize::Large,
                            "{initials}"
                        }
                        div { class: Styles::avatar_overlay, "Change" }
                    }
                    p { class: Styles::matrix_id, "{profile.matrix_id}" }
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
