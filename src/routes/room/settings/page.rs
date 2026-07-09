use super::components::NameAndRoomImage;
use crate::{
    components::{button::Button, header::Header},
    routes::room::settings::components::{
        leave_room::LeaveRoomDialog, participants::ParticipantsList,
    },
};
use dioxus::prelude::*;
use dioxus_icons::lucide::{DoorOpen, X};
use matrix_sdk::ruma::OwnedRoomId;

#[css_module("/src/routes/room/settings/page.css")]
struct Styles;

#[component]
pub fn RoomSettingsPage(id: OwnedRoomId) -> Element {
    let mut show_leave_dialog = use_signal(|| false);

    rsx! {
        div {
            Header {
                h2 { "Room information" }
                div { class: Styles::header_buttons,
                    Button {
                        onclick: move |_| {
                            *show_leave_dialog.write() = true;
                        },
                        DoorOpen {}
                    }
                    Button {
                        onclick: |_| {
                            navigator().go_back();
                        },
                        variant: crate::components::button::ButtonVariant::Outline,
                        X {}
                    }
                }
            }
            div { class: Styles::container,
                NameAndRoomImage { id: id.clone() }
                ParticipantsList { id: id.clone() }
                LeaveRoomDialog { id: id.clone(), show_leave_dialog }
            }
        }
    }
}
