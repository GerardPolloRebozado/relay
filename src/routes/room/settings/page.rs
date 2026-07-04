use super::components::NameAndRoomImage;
use crate::{
    components::{button::Button, header::Header},
    routes::room::settings::components::participants::ParticipantsList,
};
use dioxus::prelude::*;
use dioxus_icons::lucide::X;
use matrix_sdk::ruma::OwnedRoomId;

#[css_module("/src/routes/room/settings/page.css")]
struct Styles;

#[component]
pub fn RoomSettingsPage(id: OwnedRoomId) -> Element {
    rsx! {
        div {
            Header {
                h2 {"Room information"}
                Button {
                    onclick: |_| {
                      navigator().go_back();
                    },
                    variant: crate::components::button::ButtonVariant::Outline,
                    X{}
                }
            }
            div {
                class: Styles::container,
                NameAndRoomImage { id: id.clone() },
                ParticipantsList { id: id.clone() },
            }
        }
    }
}
