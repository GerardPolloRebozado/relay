use crate::routes::room::components::ChatBubble;
use crate::routes::room::message_types::image::{ImageMessage, ImagePayload};
use crate::routes::room::message_types::video::{VideoMessage, VideoPayload};
use dioxus::prelude::*;
use matrix_sdk::ruma::events::StateEventContentChange;
use matrix_sdk::ruma::events::room::message::MessageType;
use matrix_sdk_ui::timeline::{AnyOtherStateEventContentChange, MsgLikeKind, TimelineItemContent};

pub fn render_timeline_event(
    content: &TimelineItemContent,
    sender: &str,
    is_me: bool,
    time_of_event: &str,
) -> Element {
    match content {
        TimelineItemContent::MsgLike(msg_like) => match &msg_like.kind {
            MsgLikeKind::Message(msg) => {
                let bubble_content = match msg.msgtype() {
                    MessageType::Text(text) => rsx! {
                        p { "{text.body}" }
                    },
                    MessageType::Image(img) => rsx! {
                        ImageMessage { payload: ImagePayload(img.clone()) }
                    },
                    MessageType::Video(video) => rsx! {
                        VideoMessage { payload: VideoPayload(video.clone()) }
                    },
                    _ => rsx! {
                        span { "[Unsupported File]" }
                    },
                };
                rsx! {
                    ChatBubble {
                        sender: sender.to_string(),
                        is_me,
                        time_of_event: time_of_event.to_string(),
                        {bubble_content}
                    }
                }
            }
            MsgLikeKind::Sticker(_) => {
                rsx! {
                    ChatBubble {
                        sender: sender.to_string(),
                        is_me,
                        time_of_event: time_of_event.to_string(),
                        span { style: "font-style: italic; color: gray;", "[Sticker]" }
                    }
                }
            }
            MsgLikeKind::UnableToDecrypt(_) => {
                rsx! {
                    ChatBubble {
                        sender: sender.to_string(),
                        is_me,
                        time_of_event: time_of_event.to_string(),
                        span { "Unable to decrypt" }
                    }
                }
            }
            MsgLikeKind::Redacted => {
                rsx! {
                    ChatBubble {
                        sender: sender.to_string(),
                        is_me,
                        time_of_event: time_of_event.to_string(),
                        span { style: "font-style: italic; color: gray;", "Message deleted" }
                    }
                }
            }
            MsgLikeKind::Poll(_) => {
                rsx! {
                    ChatBubble {
                        sender: sender.to_string(),
                        is_me,
                        time_of_event: time_of_event.to_string(),
                        span { style: "font-style: italic; color: gray;", "[Poll]" }
                    }
                }
            }
            MsgLikeKind::LiveLocation(_) => {
                rsx! {
                    ChatBubble {
                        sender: sender.to_string(),
                        is_me,
                        time_of_event: time_of_event.to_string(),
                        span { style: "font-style: italic; color: gray;", "[Live Location]" }
                    }
                }
            }
            MsgLikeKind::Other(other) => {
                let other_text = format!("[Message-like Event: {}]", other.event_type());
                rsx! {
                    ChatBubble {
                        sender: sender.to_string(),
                        is_me,
                        time_of_event: time_of_event.to_string(),
                        span { style: "font-style: italic; color: gray;", "{other_text}" }
                    }
                }
            }
        },
        TimelineItemContent::MembershipChange(change) => {
            let change_text = format!(
                "{} membership changed: {:?}",
                change.user_id(),
                change.change()
            );
            rsx! {
                div { style: "text-align: center; color: gray; font-size: 0.875rem; font-style: italic; margin: 0.5rem 0;",
                    "{change_text}"
                }
            }
        }
        TimelineItemContent::ProfileChange(change) => {
            let prof_text = format!("{} updated their profile", change.user_id());
            rsx! {
                div { style: "text-align: center; color: gray; font-size: 0.875rem; font-style: italic; margin: 0.5rem 0;",
                    "{prof_text}"
                }
            }
        }
        TimelineItemContent::OtherState(other_state) => {
            let state_text = match other_state.content() {
                AnyOtherStateEventContentChange::RoomName(name_change) => {
                    if let StateEventContentChange::Original { content, .. } = name_change {
                        format!("{} changed the room name to '{}'", sender, content.name)
                    } else {
                        format!("{} changed the room name", sender)
                    }
                }
                AnyOtherStateEventContentChange::RoomTopic(topic_change) => {
                    if let StateEventContentChange::Original { content, .. } = topic_change {
                        format!("{} changed the topic to '{}'", sender, content.topic)
                    } else {
                        format!("{} changed the topic", sender)
                    }
                }
                AnyOtherStateEventContentChange::RoomAvatar(_) => {
                    format!("{} changed the room avatar", sender)
                }
                AnyOtherStateEventContentChange::RoomCreate(_) => {
                    format!("Room created by {}", sender)
                }
                AnyOtherStateEventContentChange::RoomEncryption(_) => {
                    "Encryption enabled".to_string()
                }
                AnyOtherStateEventContentChange::RoomPowerLevels(_) => {
                    "Room power levels updated".to_string()
                }
                AnyOtherStateEventContentChange::RoomJoinRules(_) => {
                    "Room join rules updated".to_string()
                }
                AnyOtherStateEventContentChange::RoomHistoryVisibility(_) => {
                    "Room history visibility settings updated".to_string()
                }
                AnyOtherStateEventContentChange::RoomGuestAccess(_) => {
                    "Guest access settings updated".to_string()
                }
                AnyOtherStateEventContentChange::RoomCanonicalAlias(_) => {
                    "Room alias updated".to_string()
                }
                AnyOtherStateEventContentChange::RoomPinnedEvents(_) => {
                    "Pinned events updated".to_string()
                }
                AnyOtherStateEventContentChange::RoomServerAcl(_) => {
                    "Server ACL updated".to_string()
                }
                AnyOtherStateEventContentChange::RoomThirdPartyInvite(_) => {
                    format!("{} invited a guest via third-party invite", sender)
                }
                AnyOtherStateEventContentChange::RoomTombstone(_) => "Room upgraded".to_string(),
                _ => {
                    format!(
                        "Room state updated (type: {})",
                        other_state.content().event_type()
                    )
                }
            };
            rsx! {
                div { style: "text-align: center; color: gray; font-size: 0.875rem; font-style: italic; margin: 0.5rem 0;",
                    "{state_text}"
                }
            }
        }
        TimelineItemContent::FailedToParseMessageLike { event_type, error } => {
            let parse_text = format!(
                "Failed to parse message-like event (type: {}): {}",
                event_type, error
            );
            rsx! {
                div { style: "text-align: center; color: gray; font-size: 0.875rem; font-style: italic; margin: 0.5rem 0;",
                    "{parse_text}"
                }
            }
        }
        TimelineItemContent::FailedToParseState {
            event_type, error, ..
        } => {
            let parse_text = format!(
                "Failed to parse state event (type: {}): {}",
                event_type, error
            );
            rsx! {
                div { style: "text-align: center; color: gray; font-size: 0.875rem; font-style: italic; margin: 0.5rem 0;",
                    "{parse_text}"
                }
            }
        }
        TimelineItemContent::CallInvite => {
            rsx! {
                div { style: "text-align: center; color: gray; font-size: 0.875rem; font-style: italic; margin: 0.5rem 0;",
                    "Call invite received"
                }
            }
        }
        TimelineItemContent::RtcNotification { .. } => {
            rsx! {
                div { style: "text-align: center; color: gray; font-size: 0.875rem; font-style: italic; margin: 0.5rem 0;",
                    "Call notification received"
                }
            }
        }
    }
}
