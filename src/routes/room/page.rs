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
    let mut room_name = use_signal(|| "Room".to_string());

    use_future(move || {
        let value = cloned_id.clone();
        async move {
            let state = use_context::<AppState>();
            let client = state.matrix.read().client().await.unwrap();
            let mut _room = client.get_room(&value);
            if _room.is_none() {
                error!("Could not get room");
                navigator().push(Route::Login);
                return;
            }
            if let Ok(display_name) = _room.clone().unwrap().display_name().await {
                *room_name.write() = display_name.to_room_alias_name().clone();
            }
            room.set(_room);
        }
    });

    if room.read().is_none() {
        return rsx! {"Loading..."};
    }
    let room_for_reject = room.read().clone().unwrap();
    let room_for_accept = room.read().clone().unwrap();
    let room_id = id.clone();
    rsx! {
        div {
            class: Styles::container,
            h2 { "{room_name}" }
            div { class: Styles::chat_container,
                RoomTimeline { class: Styles::message_list, room_id: id.clone() }
                if room_for_reject.state() == RoomState::Joined {
                    MessageInput {room_id: id }
                } else {
                    div {
                        class: Styles::invitation_buttons,
                        Button {
                            variant: ButtonVariant::Destructive,
                            onclick: move |_evt: MouseEvent| {
                                let room_clone = room_for_reject.clone();
                                async move {
                                    let _ = room_clone.leave().await;
                                    navigator().push(Route::Home);
                                }
                            },
                            "Reject"
                        }
                        Button {
                            onclick: move |_evt: MouseEvent| {
                                let room_clone = room_for_accept.clone();
                                let room_id_clone = room_id.clone();
                                async move {
                                    let _ = room_clone.join().await;
                                    navigator().push(Route::RoomPage { id: room_id_clone });
                                }
                            },
                            "Accept"
                        }
                    }
                }
            }
        }
    }
}
