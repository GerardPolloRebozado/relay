use matrix_sdk::{Room, ruma::OwnedRoomId};

#[derive(Clone, PartialEq, Eq)]
pub struct SpaceInfo {
    pub id: OwnedRoomId,
    pub name: String,
    pub avatar_url: String,
}

#[derive(Clone)]
pub struct RoomContainer(pub Room);

impl PartialEq for RoomContainer {
    fn eq(&self, other: &Self) -> bool {
        self.0.room_id() == other.0.room_id()
    }
}

impl RoomContainer {
    pub fn new(room: Room) -> Self {
        Self { 0: room }
    }
}
