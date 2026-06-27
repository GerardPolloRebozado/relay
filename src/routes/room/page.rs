use crate::{
    components::button::{Button, ButtonVariant},
    routes::{
        room::components::{MessageInput, RoomTimeline},
        router::Route,
    },
    state::app_state::AppState,
};
use dioxus::prelude::*;
use matrix_sdk::ruma::OwnedRoomId;
use matrix_sdk::{room::Room, RoomState};

#[css_module("/src/routes/room/page.css")]
struct Styles;

#[component]
pub fn RoomPage(id: OwnedRoomId) -> Element {
    let cloned_id = id.clone();
    let mut room = use_signal(|| None::<Room>);

    use_future(move || {
        let value = cloned_id.clone();
        async move {
            let state = use_context::<AppState>();
            let client = state.matrix.read().client().await.unwrap();
            let _room = client.get_room(&value);
            if _room.is_none() {
                navigator().push(Route::Login);
            }
            room.set(_room);
        }
    });

    if room.read().is_none() {
        return rsx! {"Loading..."};
    }

    let mut unwrapped_room = room.read().clone().unwrap();

    rsx! {
        div {
            class: Styles::container,
            h2 { "Room" }
            div { class: Styles::chat_container,
                RoomTimeline { class: Styles::message_list, room_id: id.clone() }
                if unwrapped_room.state() == RoomState::Joined {
                    MessageInput {room_id: id }
                } else {
                    div {
                        class: Styles::invitation_buttons,
                        Button {
                            variant: ButtonVariant::Destructive,
                            "Reject"
                        }
                        Button {
                            "Accept"
                        }
                    }
                }
            }
        }
    }
}
