use dioxus::prelude::*;
use futures_util::{StreamExt, pin_mut};
use matrix_sdk::{Client, Room};
use matrix_sdk_ui::room_list_service::filters::BoxedFilterFn;

use crate::{
    routes::{
        home::dm_utilities::{RoomInfo, get_last_message_in_room, get_room_avatar},
        router::Route,
    },
    state::app_state::AppState,
};

pub async fn room_list_filler(
    rooms_list: &mut Signal<Vec<RoomInfo>>,
    filters: BoxedFilterFn,
    is_loading: &mut Signal<bool>,
) {
    let state = use_context::<AppState>();

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

    controller.set_filter(filters);

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
            is_loading.set(false);
        }
    }
}

pub async fn fetch_room_info(room: Room, client: Client) -> RoomInfo {
    use matrix_sdk::ruma::events::room::message::MessageType;
    use matrix_sdk::ruma::events::{AnySyncMessageLikeEvent, AnySyncTimelineEvent};

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
            let Ok(event) = timeline_event.raw().deserialize() else {
                continue;
            };
            if let AnySyncTimelineEvent::MessageLike(AnySyncMessageLikeEvent::RoomMessage(msg)) =
                event
                && let Some(original_msg) = msg.as_original()
                && let MessageType::Text(text_msg) = &original_msg.content.msgtype
            {
                last_message = text_msg.body.clone();
                break;
            }
        }
    }
    RoomInfo {
        room,
        name,
        avatar_url,
        last_message,
    }
}
