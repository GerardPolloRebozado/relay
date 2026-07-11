use crate::components::spinner::Spinner;
use crate::routes::space::components::header::SpaceHeader;
use crate::routes::space::components::room_list::SpaceRoomListPage;
use dioxus::prelude::*;
use matrix_sdk::ruma::OwnedRoomId;

use crate::custom_types::rooms::RoomContainer;
use crate::{
    routes::router::Route,
    state::{
        app_state::AppState,
        notifications::{Notification, NotificationType, NotificationsState},
    },
};

#[component]
pub fn SpacePage(id: OwnedRoomId) -> Element {
    let state = use_context::<AppState>();
    let notifications = use_context::<NotificationsState>();

    let mut id_signal = use_signal(|| id.clone());
    let mut is_loading = use_signal(|| false);

    if *id_signal.read() != id {
        id_signal.set(id.clone());
        is_loading.set(true);
    }

    let space_resource = use_resource(move || {
        let current_id = id_signal.read().clone();
        let matrix_manager = state.matrix.read().clone();
        let mut notifications = notifications;
        let mut is_loading = is_loading;

        async move {
            let client = matrix_manager.client().await.unwrap();
            let _space = client.get_room(&current_id);

            if _space.is_none() {
                notifications.push(Notification::new(
                    "Room not found",
                    "Invalid room or not invited",
                    NotificationType::Error,
                ));
                navigator().push(Route::Home);
                return None;
            }

            is_loading.set(false);
            _space
        }
    });

    rsx! {
        div {
            if *is_loading.read() {
                Spinner {}
            } else {
                match &*space_resource.read_unchecked() {
                    Some(Some(actual_space)) => rsx! {
                        SpaceHeader { space: RoomContainer::new(actual_space.clone()) }
                        SpaceRoomListPage { space: RoomContainer::new(actual_space.clone()) }
                    },
                    Some(None) => rsx! {
                        div { "Redirecting..." }
                    },
                    None => rsx! {
                        Spinner {}
                    },
                }
            }
        }
    }
}
