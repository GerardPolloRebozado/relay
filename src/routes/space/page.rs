use crate::components::spinner::Spinner;
use crate::routes::home::dm_utilities::get_room_avatar;
use crate::routes::space::components::header::SpaceHeader;
use dioxus::prelude::*;
use matrix_sdk::{Room, ruma::OwnedRoomId};

use crate::custom_types::rooms::{RoomContainer, SpaceInfo};
use crate::{
    routes::router::Route,
    state::{
        app_state::AppState,
        notifications::{Notification, NotificationType, NotificationsState},
    },
};

#[component]
pub fn SpacePage(id: OwnedRoomId) -> Element {
    let mut space = use_signal(|| None::<Room>);
    let mut space_info = use_signal(|| None::<SpaceInfo>);

    use_future(move || {
        let cloned_id = id.clone();
        async move {
            let state = use_context::<AppState>();
            let matrix_manager = state.matrix.read().clone();
            let client = matrix_manager.client().await.unwrap();
            let _space = client.get_room(&cloned_id);
            if _space.is_none() {
                let mut notifications = use_context::<NotificationsState>();
                notifications.push(Notification::new(
                    "Room not found",
                    "Invalid room or not invited",
                    NotificationType::Error,
                ));
                navigator().push(Route::Home);
                return;
            }
            space.set(_space.clone());
            let _space = _space.unwrap();

            let display_name = _space.display_name().await;
            let name = match display_name {
                Ok(dn) => dn.to_string(),
                Err(_) => "Unknown Space".to_string(),
            };
            let avatar_url = get_room_avatar(&client, &_space)
                .await
                .unwrap_or(String::new());

            space_info.set(Some(SpaceInfo {
                id: _space.room_id().to_owned(),
                name,
                avatar_url,
            }));
        }
    });

    rsx! {
        div {
            {if space.read().is_some() {
                rsx!{
                    SpaceHeader{ space: RoomContainer::new(space.read().clone().unwrap())}
                }
            } else {
                rsx! {
                    Spinner{}
                }
            }}
        }
    }
}
