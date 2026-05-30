use crate::components::button::Button;
use crate::components::input::Input;
use crate::state::app_state::AppState;
use chrono::{TimeZone, Utc};
use dioxus::prelude::*;
use dioxus_icons::lucide::Send;
use futures_util::StreamExt;
use matrix_sdk::ruma::events::room::message::{MessageType, RoomMessageEventContent};
use matrix_sdk::ruma::OwnedRoomId;
use matrix_sdk_ui::timeline::{RoomExt, VirtualTimelineItem};
use std::sync::Arc;

#[css_module("/src/routes/room/components.css")]
struct Styles;

#[component]
pub fn RoomTimeline(
    room_id: OwnedRoomId,
    #[props(extends=GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let state = use_context::<AppState>();
    let mut messages = use_signal(Default::default);

    use_effect(move || {
        let matrix = state.matrix.read().clone();
        let room_id = room_id.clone();

        spawn(async move {
            let client = {
                let manager = matrix.read().await;
                manager.client()
            };

            let Some(client) = client else {
                eprintln!("Client not found");
                return;
            };

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
        div {
            ..attributes,

            for item in messages.read().iter().rev() {
                {
                    if let Some(event) = item.as_event() {
                        let sender = event.sender().to_string();
                        let content = event.content();

                    if content.is_unable_to_decrypt() {
                        rsx! {
                            div {
                                strong { "{sender}: " }
                                span { "Unable to decrypt" }
                            }
                        }
                    } else if let Some(msg) = content.as_message() {
                            match msg.msgtype() {
                                MessageType::Text(text) => rsx! {
                                    div {
                                        strong { "{sender}: " }
                                        span { "{text.body}" }
                                    }
                                },
                                MessageType::Image(img) => rsx! {
                                    div {
                                        strong { "{sender}: " }
                                        span {
                                            style: "font-style: italic; color: gray;",
                                            "[Image: {img.body}]"
                                        }
                                    }
                                },
                                MessageType::Video(video) => rsx! {
                                    div {
                                        strong { "{sender}: " }
                                        span {
                                            style: "font-style: italic; color: gray;",
                                            "[Video: {video.body}]"
                                        }
                                    }
                                },
                                _ => rsx! {
                                    div {
                                        strong { "{sender}: " }
                                        span {
                                            style: "font-style: italic; color: gray;",
                                            "[Unsupported File]"
                                        }
                                    }
                                },
                            }
                        } else if let Some(_sticker) = content.as_sticker() {
                            rsx! {
                                div {
                                    strong { "{sender}: " }
                                    span {
                                        style: "font-style: italic; color: gray;",
                                        "[Sticker]"
                                    }
                                }
                            }
                        } else {
                            rsx! {
                                div {
                                    style: "color: lightgray; font-size: 0.875rem;",
                                    "System event"
                                }
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

    let mut message_text = use_signal(String::new);

    rsx! {
        form {
            class: Styles::input_area,
            onsubmit: move |e: FormEvent| {
                e.prevent_default();

                let message = message_text.read().clone();

                if message.trim().is_empty() {
                    return;
                }

                message_text.set(String::new());

                let matrix = state.matrix.read().clone();
                let content = RoomMessageEventContent::text_plain(message);
                let cloned_id = room_id.clone();

                spawn(async move {
                    let client = matrix.read().await.client();
                    if let Some(client) = client {
                        if let Some(room) = client.get_room(&cloned_id) {
                            if let Err(e) = room.send(content).await {
                                error!("Failed to send message: {:?}", e);
                            }
                        } else {
                             error!("Cannot get room");
                        }
                    } else {
                        error!("Cannot get client");
                    }
                });
            },
            Input {
                r#type: "text",
                placeholder: "Type a message...",
                name: "text",

                value: "{message_text}",
                oninput: move |e: Event<FormData>| message_text.set(e.value())
            }
            Button {
                r#type: "submit",
                disabled: message_text.len() == 0,
                Send {}
            }
        }
    }
}
