use matrix_sdk::room::{Messages, MessagesOptions};
use matrix_sdk::ruma::UInt;
use matrix_sdk::ruma::api::Direction::Backward;
use matrix_sdk::{Client, Room};

pub async fn get_room_avatar(client: &Client, room: &Room) -> Option<String> {
    crate::utilities::media::get_room_avatar(client, room, 50).await
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

#[derive(Clone)]
pub struct RoomInfo {
    pub room: Room,
    pub name: String,
    pub avatar_url: String,
    pub last_message: String,
}

impl PartialEq for RoomInfo {
    fn eq(&self, other: &Self) -> bool {
        self.room.room_id() == other.room.room_id()
            && self.name == other.name
            && self.avatar_url == other.avatar_url
            && self.last_message == other.last_message
    }
}
