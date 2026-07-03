use dioxus::prelude::*;
use matrix_sdk::ruma::OwnedRoomId;

#[component]
pub fn RoomSettingsPage(id: OwnedRoomId) -> Element {
    rsx! {
        {"{id}"}
    }
}
