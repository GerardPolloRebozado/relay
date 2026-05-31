#![allow(non_snake_case)]
pub mod components;
pub mod layouts;
pub mod routes;
pub mod services;
pub mod state;

use crate::routes::router::Route;
use crate::state::secure_state::init_secure_storage;
use dioxus::prelude::*;
use log::debug;
use state::app_state::AppState;

fn main() {
    env_logger::init();
    init_secure_storage();
    launch(App);
}

fn App() -> Element {
    let mut state = use_context_provider(AppState::new);

    use_context_provider(state::notifications::NotificationsState::default);

    let dark_mode = use_signal(|| true);

    // Initial load from storage
    use_future(move || async move {
        let manager = state.matrix.cloned();
        if manager.load_from_storage().await.is_some() {
            debug!("Loaded client from storage");
            let _ = manager.start_sync().await;
        }
        state.is_loaded.set(true);
    });

    rsx! {
        Stylesheet { href: asset!("/assets/main.css") }
        div { class: if dark_mode() { "dark" } else { "" },
            div { id: "app-container",
                if !state.is_loaded.cloned() {
                    div { class: "loader-container",
                        div { class: "loader" }
                    }
                } else {
                    layouts::notifications::Notifications {}
                    Router::<Route> {}
                }
            }
        }
    }
}
