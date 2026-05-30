use crate::components::scroll_area::ScrollArea;
use crate::routes::home::components::{DMCard, DMInfo};
use crate::routes::home::dm_utilities::{get_last_message_in_room, get_room_avatar};
use crate::routes::router::Route;
use crate::state::app_state::AppState;
use dioxus::prelude::*;
use futures_util::{pin_mut, StreamExt};
use matrix_sdk::ruma::events::{AnySyncMessageLikeEvent, AnySyncTimelineEvent};
use matrix_sdk::ruma::events::room::message::MessageType;

#[css_module("/src/routes/home/page.css")]
struct Styles;

#[component]
pub fn Home() -> Element {
    let state = use_context::<AppState>();
    let mut rooms_list = use_signal(Vec::<DMInfo>::new); // (id, name, avatar)

    use_future(move || async move {
        println!("DEBUG: Home component use_future started");
        let matrix = state.matrix.read().clone();
        
        let (client, room_list_service) = {
            let manager = matrix.read().await;
            (manager.client(), manager.room_list_service())
        };

        let (Some(client), Some(room_list_service)) = (client, room_list_service) else {
            println!("DEBUG: Client or RoomListService missing, redirecting to login");
            navigator().push(Route::Login);
            return;
        };

        println!("DEBUG: Home component fetching all_rooms...");
        let all_rooms_list = match room_list_service.all_rooms().await {
            Ok(list) => list,
            Err(e) => {
                eprintln!("ERROR: Failed to get all_rooms: {:?}", e);
                return;
            }
        };
        println!("DEBUG: Home component all_rooms list obtained");

        let (room_list_stream, controller) = all_rooms_list.entries_with_dynamic_adapters(100);
        pin_mut!(room_list_stream);
        
        // Active the stream by setting a filter
        controller.set_filter(Box::new(|_| true));
        println!("DEBUG: Home component room list stream activated");

        let fetch_room_info = |room: matrix_sdk::Room, client: matrix_sdk::Client| async move {
            let display_name = room.display_name().await;
            let name = match display_name {
                Ok(dn) => dn.to_string(),
                Err(_) => "Unknown Room".to_string(),
            };
            let avatar_url = get_room_avatar(&client, &room)
                .await
                .unwrap_or_else(String::new);
            let mut last_message = String::new();
            if let Some(option_last_message) = get_last_message_in_room(&room).await {
                for timeline_event in option_last_message.chunk {
                    let Ok(event) = timeline_event.raw().deserialize() else { continue; };
                    if let AnySyncTimelineEvent::MessageLike(AnySyncMessageLikeEvent::RoomMessage(msg)) = event {
                        if let Some(original_msg) = msg.as_original() {
                            if let MessageType::Text(text_msg) = &original_msg.content.msgtype {
                                last_message = text_msg.body.clone();
                                break;
                            }
                        }
                    }
                }
            }
            DMInfo {
                room_id: room.room_id().to_owned(),
                name,
                avatar_url,
                last_message,
            }
        };

        while let Some(diffs) = room_list_stream.next().await {
            for diff in diffs {
                match diff {
                    matrix_sdk_ui::eyeball_im::VectorDiff::Reset { values } => {
                        let mut new_list = Vec::new();
                        for item in values {
                            let room = item.into_inner();
                            if room.is_dm() {
                                new_list.push(fetch_room_info(room, client.clone()).await);
                            }
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
                            if room.is_dm() {
                                let info = fetch_room_info(room, client.clone()).await;
                                rooms_list.write().push(info);
                            }
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
            header { class: "home-header",
                h2 { class: Styles::page_title, "Messages" }
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
