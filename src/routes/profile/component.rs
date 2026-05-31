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
                    let manager = state.matrix.cloned();
                    manager.logout().await;
                });
            },
            "Logout"
        }
    }
}
