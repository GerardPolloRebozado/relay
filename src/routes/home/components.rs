use crate::components::avatar::{AvatarImageSize, ImageAvatar};
use crate::components::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::routes::router::Route;
use dioxus::prelude::*;
use dioxus_router::components::Link;
use matrix_sdk::ruma::OwnedRoomId;

#[derive(Clone, PartialEq)]
pub struct DMInfo {
    pub room_id: OwnedRoomId,
    pub name: String,
    pub avatar_url: String,
    pub last_message: String,
}

#[css_module("/src/routes/home/components.css")]
struct Styles;

#[component]
pub fn DMCard(dm: DMInfo) -> Element {
    rsx! {
        Link {
            to: Route::Room {
                id: dm.room_id.clone(),
            },
            class: "no-link-highlight",
            Card { key: "{dm.room_id}",
                CardHeader { class: Styles::card_header,
                    ImageAvatar {
                        src: "{dm.avatar_url}",
                        alt: "{dm.name}",
                        size: AvatarImageSize::Medium,
                        "{dm.name.chars().next().unwrap_or('?')}"
                    }
                    CardTitle { "{dm.name}" }
                }
                CardContent {
                    CardDescription { "{dm.last_message}" }
                }
            }
        }
    }
}
