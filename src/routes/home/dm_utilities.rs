use base64::{engine::general_purpose::STANDARD, Engine as _};
use log::{debug, error};
use matrix_sdk::room::{Messages, MessagesOptions};
use matrix_sdk::ruma::api::Direction::Backward;
use matrix_sdk::ruma::media::Method;
use matrix_sdk::ruma::UInt;
use matrix_sdk::{
    media::{MediaFormat, MediaThumbnailSettings},
    Client, Room, RoomMemberships,
};

pub async fn get_room_avatar(client: &Client, room: &Room) -> Option<String> {
    let format = MediaFormat::Thumbnail(MediaThumbnailSettings {
        method: Method::Crop,
        width: 50u32.into(),
        height: 50u32.into(),
        animated: false,
    });

    if let Ok(Some(bytes)) = room.avatar(format.clone()).await {
        return encode_to_data_uri(bytes);
    }

    if let Ok(members) = room.members_no_sync(RoomMemberships::JOIN).await {
        let my_user_id = client.user_id().unwrap();

        let other_members: Vec<_> = members
            .into_iter()
            .filter(|m| m.user_id() != my_user_id)
            .collect();

        if other_members.len() == 1 {
            let other_user = &other_members[0];

            debug!("Using fallback avatar from user: {}", other_user.user_id());

            if let Ok(Some(bytes)) = other_user.avatar(format).await {
                return encode_to_data_uri(bytes);
            }
        }
    }

    None
}

fn encode_to_data_uri(bytes: Vec<u8>) -> Option<String> {
    if bytes.len() > 1_048_576 {
        error!("Avatar too large: {} bytes", bytes.len());
        return None;
    }
    let b64_string = STANDARD.encode(&bytes);
    Some(format!("data:image/png;base64,{}", b64_string))
}

pub async fn get_last_message_in_room(room: &Room) -> Option<Messages> {
    let mut last_msg_options = MessagesOptions::new(Backward);
    last_msg_options.limit = UInt::from(100u32);
    let last_msg = room.messages(last_msg_options).await;
    if last_msg.is_err() {
        return None;
    }
    Some(last_msg.unwrap())
}
