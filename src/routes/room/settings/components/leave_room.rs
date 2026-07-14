use dioxus::prelude::*;
use matrix_sdk::ruma::OwnedRoomId;

use crate::{
    components::alert_dialog::{
        AlertDialog, AlertDialogAction, AlertDialogActions, AlertDialogCancel,
        AlertDialogDescription, AlertDialogTitle,
    },
    routes::router::Route,
    state::{
        self,
        app_state::AppState,
        notifications::{self, Notification},
    },
};

#[component]
pub fn LeaveRoomDialog(id: OwnedRoomId, show_leave_dialog: Signal<bool>) -> Element {
    let state = use_context::<AppState>();
    let notifications = use_context::<notifications::NotificationsState>();

    rsx! {
        AlertDialog { open: *show_leave_dialog.read(),
            AlertDialogTitle { "Leave room" }
            AlertDialogDescription { "Are you sure you want to leave this room?" }
            AlertDialogActions {
                AlertDialogCancel { on_click: move |_| *show_leave_dialog.write() = false, "Cancel" }
                AlertDialogAction {
                    on_click: move |_| {
                        let cloned_id = id.clone();
                        let mut notifications = notifications;
                        async move {
                            let matrix_manager = state.matrix.read().clone();
                            let client = matrix_manager.client().await.unwrap();
                            let room = client.get_room(&cloned_id);
                            if room.is_none() {
                                error!("Could not get room {}", cloned_id);
                                navigator().push(Route::Home);
                                return;
                            }
                            let room = room.unwrap();
                            if room.leave().await.is_ok() {
                                if room.forget().await.is_err() {
                                    let new_notification = Notification::new(
                                        "Error forgetting the room",
                                        "You have successfully left the room, but it could not be forgotten",
                                        state::notifications::NotificationType::Error,
                                    );
                                    notifications.push(new_notification);
                                }
                                navigator().push(Route::Home);
                            } else {
                                let new_notification = Notification::new(
                                    "Error leaving the room",
                                    "An unknown error occurred while leaving the room",
                                    state::notifications::NotificationType::Error,
                                );
                                notifications.push(new_notification);
                            }
                        }
                    },
                    "Leave"
                }
            }
        }
    }
}
