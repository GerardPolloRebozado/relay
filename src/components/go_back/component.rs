use dioxus::prelude::*;
use dioxus_icons::lucide::ArrowLeft;

use crate::components::button::{Button, ButtonSize, ButtonVariant};

#[component]
pub fn GoBackButton() -> Element {
    rsx! {
        Button {
            style: "cursor: pointer;",
            size: ButtonSize::Icon,
            variant: ButtonVariant::Ghost,
            onclick: |_| navigator().go_back(),
            ArrowLeft {}
        }
    }
}
