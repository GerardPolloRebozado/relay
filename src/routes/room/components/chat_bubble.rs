use dioxus::prelude::*;

#[css_module("src/routes/room/components/chat_bubble.css")]
struct Styles;

#[component]
pub fn ChatBubble(
    sender: String,
    is_me: bool,
    time_of_event: String,
    children: Element,
) -> Element {
    let alignment_class = if is_me {
        Styles::my_message
    } else {
        Styles::others_message
    };

    rsx! {
        div { class: alignment_class,
            div { class: Styles::message,
                strong {"{sender}"  }
                div {
                {children}
                }
                div {
                    class: Styles::additional_info,
                p {
                    class: Styles::event_time,
                    {time_of_event}
                }
                }
            }
        }
    }
}
