use crate::components::button::{Button, ButtonVariant};
use crate::components::card::Card;
use crate::routes::router::Route;
use crate::state::app_state::AppState;
use dioxus::prelude::*;

#[css_module("src/routes/settings/components/account.css")]
struct Styles;

#[component]
pub fn AccountCard() -> Element {
    let state = use_context::<AppState>();

    let onlogout = move |_| {
        let matrix = state.matrix.cloned();
        spawn(async move {
            matrix.logout().await;
            navigator().push(Route::Login);
        });
    };

    rsx! {
        Card { class: Styles::settings_card,
            div { class: Styles::account_section,
                h3 { class: Styles::account_title, "Account" }
                p { class: Styles::account_description,
                    "Log out of your current Matrix session on this device."
                }
                Button {
                    variant: ButtonVariant::Destructive,
                    class: Styles::logout_button,
                    onclick: onlogout,
                    "Log Out"
                }
            }
        }
    }
}
