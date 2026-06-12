use crate::components::button::Button;
use crate::components::input::Input;
use crate::components::label::Label;
use crate::routes::room::message_types::image::ImageMessage;
use crate::routes::room::message_types::image::ImagePayload;
use crate::state::app_state::AppState;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::{DateTime, Local, TimeZone, Utc};
use dioxus::html::FileData;
use dioxus::prelude::*;
use dioxus_icons::lucide::Plus;
use dioxus_icons::lucide::Send;
use futures_util::StreamExt;
use matrix_sdk::media::MediaFormat;
use matrix_sdk::media::MediaRequestParameters;
use matrix_sdk::ruma::events::room::message::{MessageType, RoomMessageEventContent};
use matrix_sdk::ruma::events::AnyMessageLikeEventContent;
use matrix_sdk::ruma::{OwnedRoomId, OwnedUserId};
use matrix_sdk_ui::eyeball_im::Vector;
use matrix_sdk_ui::timeline::AttachmentSource;
use matrix_sdk_ui::timeline::{AttachmentConfig, RoomExt, TimelineItem, VirtualTimelineItem};
use mime_guess;
use std::sync::Arc;

#[css_module("/src/routes/room/components.css")]
struct Styles;

#[component]
fn ChatBubble(sender: String, is_me: bool, time_of_event: String, children: Element) -> Element {
    let alignment_class = if is_me {
        Styles::my_message
    } else {
        Styles::others_message
    };

    rsx! {
        div { class: alignment_class,
            div { class: Styles::message,
                strong { class: Styles::sender ,"{sender}"  }
                div {
                {children}
                }
                div {
                    class: Styles::additional_info,
                p {
                    class: Styles::event_time,
                    {time_of_event}
                }
                }
            }
        }
    }
}

#[component]
pub fn RoomTimeline(
    room_id: OwnedRoomId,
    #[props(extends=GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let state = use_context::<AppState>();
    let mut messages = use_signal(Vector::<Arc<TimelineItem>>::default);
    let mut current_user_id = use_signal(|| None::<OwnedUserId>);

    use_effect(move || {
        let matrix = state.matrix.cloned();
        let room_id = room_id.clone();

        spawn(async move {
            let client = matrix.client().await;

            let Some(client) = client else {
                eprintln!("Client not found");
                return;
            };

            current_user_id.set(client.user_id().map(|id| id.to_owned()));

            let Some(room) = client.get_room(&room_id) else {
                eprintln!("Room not found");
                return;
            };

            let timeline = Arc::new(room.timeline_builder().build().await.unwrap());

            let (initial_items, mut stream) = timeline.subscribe().await;
            messages.set(initial_items);

            let timeline_clone = timeline.clone();
            spawn(async move {
                if let Err(e) = timeline_clone.paginate_backwards(20).await {
                    eprintln!("Error paginating backwards: {:?}", e);
                }
            });

            while let Some(diff_batch) = stream.next().await {
                let mut msgs = messages.write();
                for diff in diff_batch {
                    diff.apply(&mut *msgs);
                }
            }
        });
    });

    rsx! {
        div { ..attributes,

            for item in messages.read().iter().rev() {
                {
                    if let Some(event) = item.as_event() {
                        let sender = event.sender().to_string();
                        let content = event.content();

                        let timestamp = event.timestamp();
                        let datetime: DateTime<Local> = Local.timestamp_millis_opt(timestamp.0.into()).single().unwrap();
                        let time_of_event = datetime.format("%H:%M").to_string();

                        let is_me = current_user_id
                            .read()
                            .as_ref()
                            .map(|id| id.as_str() == sender)
                            .unwrap_or(false);

                        if content.as_message().is_some()
                            || content.is_unable_to_decrypt()
                            || content.as_sticker().is_some()
                        {
                            rsx! {
                                ChatBubble { sender, is_me, time_of_event,
                                    {
                                        if let Some(msg) = content.as_message() {
                                            match msg.msgtype() {
                                                MessageType::Text(text) => rsx! { span { "{text.body}" } },
                                                MessageType::Image(img) => {
                                                    rsx! {
                                                    ImageMessage {
                                                            payload: ImagePayload(img.clone())
                                                        }
                                                    }
                                                },
                                                MessageType::Video(video) => rsx! {
                                                    span {
                                                        "[Video: {video.body}]"
                                                    }
                                                },
                                                _ => rsx! {
                                                    span {
                                                        "[Unsupported File]"
                                                    }
                                                },
                                            }
                                        } else if content.is_unable_to_decrypt() {
                                            rsx! { span { "Unable to decrypt" } }
                                        } else if let Some(_sticker) = content.as_sticker() {
                                            rsx! {
                                                span {
                                                    style: "font-style: italic; color: gray;",
                                                    "[Sticker]"
                                                }
                                            }
                                        } else {
                                            rsx! { "" }
                                        }
                                    }
                                }
                            }
                        } else {
                            rsx! {
                                div { style: "color: lightgray; font-size: 0.875rem;", "System event" }
                            }
                        }
                    } else if item.is_date_divider() {
                        let date_text = if let Some(VirtualTimelineItem::DateDivider(ts)) = item.as_virtual() {
                            Utc.timestamp_millis_opt(ts.0.into())
                                .single()
                                .map(|d| d.format("%B %e, %Y").to_string())
                                .unwrap_or_else(|| "Unknown Date".to_string())
                        } else {
                            "Date Divider".to_string()
                        };
                        rsx! {
                            div {
                                style: "text-align: center; color: gray; font-size: 0.875rem; margin: 1rem 0;",
                                "{date_text}"
                            }
                        }
                    } else {
                        rsx! {
                            div {
                                style: "color: lightgray; font-size: 0.875rem;",
                                "Other event"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn MessageInput(
    #[props(extends=GlobalAttributes)] attributes: Vec<Attribute>,
    room_id: OwnedRoomId,
) -> Element {
    let state = use_context::<AppState>();

    let mut text_input = use_signal(String::new);
    let mut file_input_key = use_signal(|| 0);
    let mut selected_files = use_signal(Vec::<FileData>::new);

    rsx! {
        form {
            class: Styles::input_area,
            onsubmit: move |e: FormEvent| {
                e.prevent_default();
                let text_to_send = text_input();
                let files_to_send = selected_files();

                if text_to_send.trim().is_empty() && files_to_send.is_empty() {
                    return;
                }

                text_input.set(String::new());
                selected_files.set(Vec::new());
                file_input_key.with_mut(|k| *k += 1);

                let matrix = state.matrix.cloned();
                let room_id = room_id.clone();

                spawn(async move {
                    let client = matrix.client().await.unwrap();
                    let room = client.get_room(&room_id).unwrap();
                    let timeline = room.timeline().await.unwrap();
                    if !text_to_send.trim().is_empty() {
                        let _ = timeline.send(AnyMessageLikeEventContent::RoomMessage(RoomMessageEventContent::text_plain(text_to_send)));
                    }

                    for file in files_to_send {
                        let file_name = file.name();
                        if let Ok(bytes) = file.read_bytes().await {
                             let mime_type = mime_guess::from_path(std::path::Path::new(&file_name)).first_or_octet_stream();
                             let source = AttachmentSource::Data {
                                 filename: file_name,
                                 bytes: bytes.to_vec(),
                             };

                             let config = AttachmentConfig::default();
                             let _ = timeline.send_attachment(source, mime_type, config).await;
                        }
                    }
                });
            },
            div {
                class: Styles::file_input_wrapper,
                Input {
                    key: "{file_input_key}",
                    r#type: "file",
                    name: "file",
                    id: "file",
                    class: Styles::file_input,
                    onchange: move |e: FormEvent| {
                        for file in e.files() {
                            selected_files.write().push(file);
                        }
                    }
                },
                Label {
                    html_for: "file",
                    div {
                        class: Styles::plus_button,
                        Plus {}
                    }
                }
            },
            Input {
                r#type: "text",
                placeholder: "Type a message...",
                name: "text",
                value: "{text_input}",
                oninput: move |e: Event<FormData>| text_input.set(e.value())
            }
            Button {
                r#type: "submit",
                disabled: text_input().is_empty() && selected_files().is_empty(),
                Send {}
            }
        }
    }
}
