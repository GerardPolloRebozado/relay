use dioxus::prelude::*;
use matrix_sdk::ruma::OwnedRoomId;

use crate::{routes::router::Route, state::app_state::AppState};

#[css_module("src/routes/room/components/header.css")]
struct Styles;

#[component]
pub fn RoomHeader(room_id: OwnedRoomId) -> Element {
    let mut room_name = use_signal(|| "Room".to_string());
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
        }
    });

    rsx! {
        div {
            class: Styles::name_image,
            onclick: move |_evt: MouseEvent| {
                navigator()
                    .push(Route::RoomSettingsPage {
                        id: cloned_room_id.clone(),
                    });
            },
            GoBackButton {}
            h2 { "{room_name}" }
        }
    }
}
