use crate::components::notification::NotificationCard;
use crate::state::notifications::NotificationsState;
use dioxus::prelude::*;

#[css_module("/src/layouts/notifications/style.css")]
pub struct Styles;

#[component]
pub fn Notifications() -> Element {
    let mut state = use_context::<NotificationsState>();

    rsx! {
        div { class: Styles::notifications_container,
            for notification in state.notifications.read().iter().cloned() {
                NotificationCard {
                    key: "{notification.id}",
                    notification: notification.clone(),
                    on_close: move |_| state.remove(notification.id),
                }
            }
        }
    }
}
