use crate::components::spinner::Spinner;
use crate::routes::home::components::{NewRoom, RoomCard};
use crate::state::app_state::AppState;
use crate::utilities::room::room_list_filler;
use dioxus::prelude::*;
use matrix_sdk::ruma::OwnedRoomId;
use matrix_sdk_ui::room_list_service::filters::{new_filter_identifiers, new_filter_not};
use matrix_sdk_ui::spaces::SpaceService;

#[css_module("/src/routes/home/page.css")]
struct Styles;

#[component]
pub fn Home() -> Element {
    let mut state = use_context::<AppState>();
    let has_cached = !state.rooms_list.read().is_empty();
    let mut rooms_list = use_signal(move || state.rooms_list.read().clone()); // (id, name, avatar)
    let mut is_loading = use_signal(move || !has_cached);

    use_future(move || async move {
        let matrix = state.matrix.cloned();

        let client = matrix.client().await.unwrap();

        // show anything but spaces and its group rooms
        let space_service = SpaceService::new(client.clone()).await;
        let space_filters = space_service.space_filters().await;
        let all_space_descendants: Vec<OwnedRoomId> = space_filters
            .iter()
            .flat_map(|filter| filter.descendants.clone())
            .collect();
        room_list_filler(
            &mut rooms_list,
            Box::new(new_filter_not(Box::new(new_filter_identifiers(
                all_space_descendants,
            )))),
            &mut is_loading,
        )
        .await;
        is_loading.set(false);
    });

    use_effect(move || {
        let list = rooms_list.read().clone();
        state.rooms_list.set(list);
    });

    use_effect(move || {
        let loading = is_loading.cloned();
        state.is_rooms_loading.set(loading);
    });

    rsx! {
        div { class: Styles::home_container,
            header { class: Styles::home_header,
                h2 { "Messages" }
                NewRoom {}
            }
            {
                if *is_loading.read() {
                    rsx! {
                        div { class: "center", Spinner {} }
                    }
                } else {
                    rsx! {
                        div { class: Styles::room_list,
                            if rooms_list.read().is_empty() {
                                div { class: Styles::empty_state,
                                    p { "No conversations found." }
                                }
                            } else {
                                for dminfo in rooms_list.read().iter() {
                                    RoomCard { roomInfo: dminfo.clone() }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
