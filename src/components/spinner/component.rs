use dioxus::prelude::*;

#[css_module("src/components/spinner/style.css")]
struct Styles;

#[component]
pub fn Spinner() -> Element {
    rsx! {
        div { class: Styles::spinner, role: "status" }
    }
}
