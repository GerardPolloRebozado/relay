use crate::components::scroll_area::ScrollArea;
use crate::components::spinner::Spinner;
use crate::routes::home::components::{DMCard, NewRoom};
use crate::routes::home::dm_utilities::{DMInfo, fetch_room_info};
use crate::routes::router::Route;
use crate::state::app_state::AppState;
use dioxus::prelude::*;
use futures_util::{StreamExt, pin_mut};
use matrix_sdk::ruma::OwnedRoomId;
use matrix_sdk_ui::room_list_service::filters::{new_filter_identifiers, new_filter_not};
use matrix_sdk_ui::spaces::SpaceService;

#[css_module("/src/routes/home/page.css")]
struct Styles;

#[component]
pub fn Home() -> Element {
    let state = use_context::<AppState>();
    let mut rooms_list = use_signal(Vec::<DMInfo>::new); // (id, name, avatar)
    let mut is_loading = use_signal(|| true);

    use_future(move || async move {
        let matrix = state.matrix.cloned();

        let (client, room_list_service) = (matrix.client().await, matrix.room_list_service().await);

        let (Some(client), Some(room_list_service)) = (client, room_list_service) else {
            navigator().push(Route::Login);
            return;
        };
        let all_rooms_list = match room_list_service.all_rooms().await {
            Ok(list) => list,
            Err(e) => {
                error!("Failed to get all_rooms: {:?}", e);
                return;
            }
        };
        let (room_list_stream, controller) = all_rooms_list.entries_with_dynamic_adapters(10);
        pin_mut!(room_list_stream);

        // show anything but spaces and its group rooms
        let space_service = SpaceService::new(client.clone()).await;
        let space_filters = space_service.space_filters().await;
        let all_space_descendants: Vec<OwnedRoomId> = space_filters
            .iter()
            .flat_map(|filter| filter.descendants.clone())
            .collect();
        controller.set_filter(Box::new(new_filter_not(Box::new(new_filter_identifiers(
            all_space_descendants,
        )))));

        while let Some(diffs) = room_list_stream.next().await {
            for diff in diffs {
                match diff {
                    matrix_sdk_ui::eyeball_im::VectorDiff::Reset { values } => {
                        let mut new_list = Vec::new();
                        for item in values {
                            let room = item.into_inner();
                            new_list.push(fetch_room_info(room, client.clone()).await);
                        }
                        rooms_list.set(new_list);
                        *is_loading.write() = false;
                    }
                    matrix_sdk_ui::eyeball_im::VectorDiff::PushBack { value } => {
                        let room = value.into_inner();
                        if room.is_dm() {
                            let info = fetch_room_info(room, client.clone()).await;
                            rooms_list.write().push(info);
                            *is_loading.write() = false;
                        }
                    }
                    matrix_sdk_ui::eyeball_im::VectorDiff::Append { values } => {
                        for item in values {
                            let room = item.into_inner();
                            let info = fetch_room_info(room, client.clone()).await;
                            rooms_list.write().push(info);
                            *is_loading.write() = false;
                        }
                    }
                    matrix_sdk_ui::eyeball_im::VectorDiff::Clear => {
                        rooms_list.set(Vec::new());
                    }
                    _ => {}
                }
            }
        }
    });

    rsx! {
            div { class: Styles::home_container,
                header { class: Styles::home_header,
                    h2 { "Messages" }
                    NewRoom{}
                }
                {if *is_loading.read() {
                    rsx!{
                        div {
                            class: "center",
                        Spinner{}
                        }}
                } else {
                    rsx!{
                    ScrollArea { class: "room-list-scroll",
                        div { class: Styles::room_list,
                            if !*state.first_sync_done.read() {
                                div { class: Styles::empty_state,
                                    div { class: "loader" }
                                    p { "Loading conversations..." }
                                }
                            } else if rooms_list.read().is_empty() {
                                div { class: Styles::empty_state,
                                    p { "No conversations found." }
                                }
                            } else {
                                for dminfo in rooms_list.read().iter() {
                                    DMCard { dm: dminfo.clone() }
                                }
                            }
                        }
                    }
                }
                }
            }
        }
    }
}
