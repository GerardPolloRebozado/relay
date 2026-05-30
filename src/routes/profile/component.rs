use crate::components::button::Button;
use crate::state::app_state::AppState;
use dioxus::prelude::*;

#[component]
pub fn Profile() -> Element {
    let state = use_context::<AppState>();
    rsx! {
        Button {
            onclick: move |_| {
                spawn(async move {
                    let matrix = state.matrix.read().clone();
                    let mut manager = matrix.write().await;
                    manager.logout().await;
                });
            },
            "Logout"
        }
    }
}
