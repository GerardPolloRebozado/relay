use dioxus::prelude::*;
use matrix_sdk_ui::{room_list_service::filters::new_filter_identifiers, spaces::SpaceService};

use crate::{
    components::{scroll_area::ScrollArea, spinner::Spinner},
    custom_types::rooms::RoomContainer,
    routes::home::{components::RoomCard, dm_utilities::RoomInfo},
    state::app_state::AppState,
    utilities::room::room_list_filler,
};

#[css_module("src/routes/space/components/room_list.css")]
struct Styles;

#[component]
pub fn SpaceRoomListPage(space: RoomContainer) -> Element {
    let state = use_context::<AppState>();
    let mut rooms_list = use_signal(Vec::<RoomInfo>::new);
    let mut is_loading = use_signal(|| true);
    let space_id = space.0.room_id().to_owned();

    use_future(move || {
        let space_id_cloned = space_id.clone();
        async move {
            let matrix = state.matrix.cloned();

            let client = matrix.client().await.unwrap();

            let space_service = SpaceService::new(client.clone()).await;
            let space_filters = space_service.space_filters().await;
            let space_filter = space_filters
                .iter()
                .find(|filter| filter.space_room.room_id == space_id_cloned);

            if let Some(filter) = space_filter {
                room_list_filler(
                    &mut rooms_list,
                    Box::new(new_filter_identifiers(filter.descendants.clone())),
                    &mut is_loading,
                )
                .await;
            }
            is_loading.set(false);
        }
    });

    {
        if *is_loading.read() {
            rsx! {
                div { class: "center", Spinner {} }
            }
        } else {
            rsx! {
                div {
                    class: Styles::room_list,
                ScrollArea {
                        if rooms_list.read().is_empty() {
                            div {
                                p { "No conversations found." }
                            }
                        } else {
                            for dminfo in rooms_list.read().iter() {
                                RoomCard { dm: dminfo.clone() }
                            }
                    }
                }
                }
            }
        }
    }
}
