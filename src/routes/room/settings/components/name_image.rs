use dioxus::prelude::*;
use matrix_sdk::ruma::OwnedRoomId;

use crate::{
    components::avatar::ImageAvatar,
    routes::{home::dm_utilities::get_room_avatar, router::Route},
    state::app_state::AppState,
};

#[derive(Default)]
struct RoomInformation {
    pub name: String,
    pub image_url: String,
}

impl RoomInformation {
    fn default() -> Self {
        RoomInformation {
            name: "Room".to_string(),
            image_url: "/assets/user.svg".to_string(),
        }
    }
}

#[css_module("src/routes/room/settings/components/name_image.css")]
struct Styles;

#[component]
pub fn NameAndRoomImage(id: OwnedRoomId) -> Element {
    let mut room_info = use_signal(RoomInformation::default);

    use_future(move || {
        let cloned_id = id.clone();
        async move {
            let state = use_context::<AppState>();
            let matrix_manager = state.matrix.read().clone();
            let client = matrix_manager.client().await.unwrap();
            let room = client.get_room(&cloned_id);
            if room.is_none() {
                error!("Could not get room {}", cloned_id);
                navigator().push(Route::Home);
                return;
            }
            let room = room.unwrap();
            if let Ok(display_name) = room.display_name().await {
                room_info.write().name = display_name.to_string();
            }
            if let Some(room_image) = get_room_avatar(&client, &room).await {
                room_info.write().image_url = room_image;
            }
        }
    });

    rsx! {
        div { class: Styles::name_and_image,
            ImageAvatar {
                src: &room_info.read().image_url,
                size: crate::components::avatar::AvatarImageSize::Large,
            }
            "{room_info.read().name}"
        }
    }
}
