use crate::routes::room::components::{MessageInput, RoomTimeline};
use dioxus::prelude::*;
use matrix_sdk::ruma::OwnedRoomId;

#[css_module("/src/routes/room/page.css")]
struct Styles;

#[component]
pub fn Room(id: OwnedRoomId) -> Element {
    rsx! {
        div {
            h2 { "Room" }
            div { class: Styles::chat_container,
                RoomTimeline { class: Styles::message_list, room_id: id.clone() }
                MessageInput {room_id: id }
            }
        }
    }
}
