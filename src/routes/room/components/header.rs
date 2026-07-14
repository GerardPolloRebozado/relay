use dioxus::prelude::*;
use matrix_sdk::ruma::OwnedRoomId;

use crate::components::avatar::{AvatarImageSize, AvatarShape, ImageAvatar};
use crate::routes::home::dm_utilities::get_room_avatar;
use crate::utilities::room::room_initials;
use crate::{routes::router::Route, state::app_state::AppState};

use crate::components::go_back::GoBackButton;

#[css_module("src/routes/room/components/header.css")]
struct Styles;

#[component]
pub fn RoomHeader(room_id: OwnedRoomId) -> Element {
    let mut room_name = use_signal(|| "Room".to_string());
    let mut room_avatar_url = use_signal(String::new);
    let cloned_room_id = room_id.clone();

    use_future(move || {
        let value = room_id.clone();
        async move {
            let state = use_context::<AppState>();
            let matrix_manager = state.matrix.read().clone();
            let client = matrix_manager.client().await.unwrap();
            let room = client.get_room(&value);
            if room.is_none() {
                error!("Could not get room");
                navigator().push(Route::Login);
                return;
            }
            if let Ok(display_name) = room.clone().unwrap().display_name().await {
                *room_name.write() = display_name.to_string();
            }
            if let Some(avatar_url) = get_room_avatar(&client, &room.clone().unwrap()).await {
                room_avatar_url.set(avatar_url);
            }
        }
    });

    rsx! {
        div { class: Styles::header_items,
            GoBackButton {}
            div {
                class: Styles::name_image,
                onclick: move |_evt: MouseEvent| {
                    navigator()
                        .push(Route::RoomSettingsPage {
                            id: cloned_room_id.clone(),
                        });
                },
                ImageAvatar {
                    size: AvatarImageSize::Small,
                    shape: AvatarShape::Rounded,
                    src: room_avatar_url,
                    {room_initials(room_name.read().to_string())}
                }
                h4 { "{room_name}" }
            }
        }
    }
}
