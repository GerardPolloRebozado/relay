use crate::layouts::encryption::Encryption;
use crate::layouts::sidebar::Sidebar;
use crate::routes::home::Home;
use crate::routes::login::Login;
use crate::routes::room::RoomPage;
use crate::routes::room::settings::page::RoomSettingsPage;
use crate::routes::settings::page::Settings;
use crate::routes::space::page::SpacePage;
use dioxus::prelude::*;
use matrix_sdk::ruma::OwnedRoomId;

#[derive(Clone, Debug, PartialEq, Routable)]
pub enum Route {
    #[route("/login")]
    Login,

    #[layout(Sidebar)]
    #[layout(Encryption)]
    #[route("/")]
    Home,

    #[route("/settings")]
    Settings,

    #[route("/room/:id")]
    RoomPage { id: OwnedRoomId },

    #[route("/room/:id/settings")]
    RoomSettingsPage { id: OwnedRoomId },

    #[route("/space/:id")]
    SpacePage { id: OwnedRoomId },
}

#[component]
fn User(id: u32) -> Element {
    rsx! { "User page for user with id: {id}" }
}
