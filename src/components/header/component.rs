use dioxus::prelude::*;

#[css_module("/src/components/header/styles.css")]
struct Styles;

#[component]
pub fn Header(
    #[props(extends=GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    rsx! {
        div {
            class: Styles::header,
            ..attributes,
            {children}
        }
    }
}
