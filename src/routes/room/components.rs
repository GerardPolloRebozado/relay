use crate::components::button::Button;
use crate::components::input::Input;
use crate::state::app_state::AppState;
use chrono::{DateTime, Local, TimeZone, Utc};
use dioxus::prelude::*;
use dioxus_icons::lucide::Send;
use futures_util::StreamExt;
use log::error;
use matrix_sdk::ruma::events::room::message::{MessageType, RoomMessageEventContent};
use matrix_sdk::ruma::{OwnedRoomId, OwnedUserId};
use matrix_sdk_ui::eyeball_im::Vector;
use matrix_sdk_ui::timeline::{RoomExt, TimelineItem, VirtualTimelineItem};
use std::sync::Arc;
use uuid::timestamp;

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

                        //get event hour and minute
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
                                                MessageType::Image(img) => rsx! {
                                                    span {
                                                        "[Image: {img.body}]"
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

                let matrix = state.matrix.cloned();
                let content = RoomMessageEventContent::text_plain(message);
                let cloned_id = room_id.clone();

                spawn(async move {
                    let client = matrix.client().await;
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
