use crate::components::scroll_area::ScrollArea;
use crate::routes::home::components::{DMCard, NewRoom};
use crate::routes::home::dm_utilities::{fetch_room_info, DMInfo};
use crate::routes::router::Route;
use crate::state::app_state::AppState;
use dioxus::prelude::*;
use futures_util::{pin_mut, StreamExt};

#[css_module("/src/routes/home/page.css")]
struct Styles;

#[component]
pub fn Home() -> Element {
    let state = use_context::<AppState>();
    let mut rooms_list = use_signal(Vec::<DMInfo>::new); // (id, name, avatar)

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
        // TODO: Make this value dynamic depending on screen size or whatever
        let (room_list_stream, controller) = all_rooms_list.entries_with_dynamic_adapters(100);
        pin_mut!(room_list_stream);

        // Active the stream by setting a filter
        controller.set_filter(Box::new(|_| true));

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
                    }
                    matrix_sdk_ui::eyeball_im::VectorDiff::PushBack { value } => {
                        let room = value.into_inner();
                        if room.is_dm() {
                            let info = fetch_room_info(room, client.clone()).await;
                            rooms_list.write().push(info);
                        }
                    }
                    matrix_sdk_ui::eyeball_im::VectorDiff::Append { values } => {
                        for item in values {
                            let room = item.into_inner();
                            let info = fetch_room_info(room, client.clone()).await;
                            rooms_list.write().push(info);
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
