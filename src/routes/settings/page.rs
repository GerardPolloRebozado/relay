use crate::routes::settings::components::{AccountCard, ProfileCard};
use dioxus::prelude::*;

#[css_module("src/routes/settings/page.css")]
struct Styles;

#[component]
pub fn Settings() -> Element {
    rsx! {
        div { class: Styles::settings_container,
            header { class: Styles::settings_header,
                h2 { "Settings" }
            }
            ProfileCard {}
            AccountCard {}
        }
    }
}
