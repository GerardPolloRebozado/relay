use crate::layouts::encryption::Encryption;
use crate::layouts::sidebar::Sidebar;
use crate::routes::home::Home;
use crate::routes::login::Login;
use crate::routes::profile::Profile;
use crate::routes::room::Room;
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

    #[route("/profile")]
    Profile,

    #[route("/room/:id")]
    Room {
        id: OwnedRoomId,
    },
}

#[component]
fn User(id: u32) -> Element {
    rsx! { "User page for user with id: {id}" }
}
