use crate::components::card::*;
use crate::layouts::notifications::Styles;
use crate::state::notifications::{Notification, NotificationAction, NotificationType};
use dioxus::prelude::*;

#[component]
pub fn NotificationCard(notification: Notification, on_close: EventHandler<()>) -> Element {
    let type_class = match notification.notif_type {
        NotificationType::Info => Styles::notif_info.inner,
        NotificationType::Success => Styles::notif_success.inner,
        NotificationType::Error => Styles::notif_error.inner,
        NotificationType::Warning => Styles::notif_warning.inner,
    };

    rsx! {
        div { onclick: move |_| on_close.call(()),
            Card { class: format!("{} {}", Styles::notification_card, type_class),
                CardHeader { class: Styles::notification_header,
                    div { class: "notification-content",
                        CardTitle { class: "notification-title", "{notification.title}" }
                        CardDescription { class: Styles::notification_message, "{notification.message}" }
                    }
                    button { class: Styles::notification_close,
                        span { class: "sr_only", "Close" }
                        svg {
                            class: "icon_sm",
                            fill: "currentColor",
                            view_box: "0 0 20 20",
                            path { d: "M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" }
                        }
                    }
                }
                if !notification.actions.is_empty() {
                    CardFooter { class: Styles::notification_actions,
                        for action in notification.actions {
                            match action {
                                NotificationAction::Button { label, on_click } => rsx! {
                                    button { class: Styles::btn_link, onclick: move |_| on_click.call(()), "{label}" }
                                },
                                NotificationAction::Link { label, route } => rsx! {
                                    Link { to: route, class: Styles::btn_link.to_string(), "{label}" }
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}
