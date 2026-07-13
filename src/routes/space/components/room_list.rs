use dioxus::prelude::*;
use matrix_sdk_ui::{room_list_service::filters::new_filter_identifiers, spaces::SpaceService};

use crate::{
    components::{scroll_area::ScrollArea, spinner::Spinner},
    custom_types::rooms::RoomContainer,
    routes::home::components::RoomCard,
    state::app_state::AppState,
    utilities::room::room_list_filler,
};

#[css_module("src/routes/space/components/room_list.css")]
struct Styles;

#[component]
pub fn SpaceRoomListPage(space: RoomContainer) -> Element {
    let mut state = use_context::<AppState>();
    let space_id = space.0.room_id().to_owned();

    let has_cached = state.space_rooms_map.read().contains_key(&space_id);
    let space_id_for_init = space_id.clone();
    let mut rooms_list = use_signal(move || {
        state.space_rooms_map.read().get(&space_id_for_init).cloned().unwrap_or_default()
    });
    let mut is_loading = use_signal(move || !has_cached);

    let space_id_for_signal = space_id.clone();
    let mut space_id_signal = use_signal(move || space_id_for_signal.clone());
    if *space_id_signal.read() != space_id {
        let new_id = space_id.clone();
        space_id_signal.set(new_id.clone());
        let new_has_cached = state.space_rooms_map.read().contains_key(&new_id);
        rooms_list.set(state.space_rooms_map.read().get(&new_id).cloned().unwrap_or_default());
        is_loading.set(!new_has_cached);
    }

    use_future(move || async move {
        let current_id = space_id_signal.read().clone();
        let matrix = state.matrix.cloned();

        let client = matrix.client().await.unwrap();

        let space_service = SpaceService::new(client.clone()).await;
        let space_filters = space_service.space_filters().await;
        let space_filter = space_filters
            .iter()
            .find(|filter| filter.space_room.room_id == current_id);

        if let Some(filter) = space_filter {
            room_list_filler(
                &mut rooms_list,
                Box::new(new_filter_identifiers(filter.descendants.clone())),
                &mut is_loading,
            )
            .await;
        }
        is_loading.set(false);
    });

    use_effect(move || {
        let current_id = space_id_signal.read().clone();
        let list = rooms_list.read().clone();
        state.space_rooms_map.write().insert(current_id, list);
    });

    {
        if *is_loading.read() {
            rsx! {
                div { class: "center", Spinner {} }
            }
        } else {
            rsx! {
                div { class: Styles::room_list,
                    ScrollArea {
                        if rooms_list.read().is_empty() {
                            div {
                                p { "No conversations found." }
                            }
                        } else {
                            for room in rooms_list.read().iter() {
                                RoomCard { roomInfo: room.clone() }
                            }
                        }
                    }
                }
            }
        }
    }
}
