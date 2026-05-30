// create global notifications state, vector of notification card components

use crate::routes::router::Route;
use dioxus::prelude::*;
use uuid::Uuid;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NotificationType {
    Info,
    Success,
    Error,
    Warning,
}

#[derive(Clone, PartialEq)]
pub enum NotificationAction {
    Button {
        label: String,
        on_click: Callback<()>,
    },
    Link {
        label: String,
        route: Route,
    },
}

#[derive(Clone, PartialEq)]
pub struct Notification {
    pub id: Uuid,
    pub title: String,
    pub message: String,
    pub notif_type: NotificationType,
    pub actions: Vec<NotificationAction>,
}

impl Notification {
    pub fn new(title: &str, message: &str, notif_type: NotificationType) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title.to_string(),
            message: message.to_string(),
            notif_type,
            actions: Vec::new(),
        }
    }

    pub fn with_action(mut self, action: NotificationAction) -> Self {
        self.actions.push(action);
        self
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
pub struct NotificationsState {
    pub notifications: Signal<Vec<Notification>>,
}

impl NotificationsState {
    pub fn push(&mut self, notification: Notification) {
        self.notifications.write().push(notification);
    }

    pub fn remove(&mut self, id: Uuid) {
        self.notifications.write().retain(|n| n.id != id);
    }
}
