use crate::services::matrix::client_manager::{MatrixEvent, MatrixManager};
use crate::custom_types::rooms::SpaceInfo;
use crate::routes::home::dm_utilities::RoomInfo;
use dioxus::prelude::*;
use matrix_sdk::ruma::OwnedRoomId;
use matrix_sdk_ui::room_list_service::State as RoomListState;
use matrix_sdk_ui::sync_service::State as SyncState;
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub struct AppState {
    pub matrix: Signal<MatrixManager>,
    pub is_syncing: Signal<bool>,
    pub is_loaded: Signal<bool>,
    pub first_sync_done: Signal<bool>,
    pub room_list_state: Signal<RoomListState>,
    pub rooms_list: Signal<Vec<RoomInfo>>,
    pub is_rooms_loading: Signal<bool>,
    pub space_list: Signal<Vec<SpaceInfo>>,
    pub is_spaces_loading: Signal<bool>,
    pub space_rooms_map: Signal<HashMap<OwnedRoomId, Vec<RoomInfo>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let (manager, mut rx) = MatrixManager::new();

        let state = Self {
            matrix: Signal::new(manager),
            is_syncing: Signal::new(false),
            is_loaded: Signal::new(false),
            first_sync_done: Signal::new(false),
            room_list_state: Signal::new(RoomListState::Init),
            rooms_list: Signal::new(Vec::new()),
            is_rooms_loading: Signal::new(true),
            space_list: Signal::new(Vec::new()),
            is_spaces_loading: Signal::new(true),
            space_rooms_map: Signal::new(HashMap::new()),
        };

        let mut is_syncing_signal = state.is_syncing;
        let mut first_sync_done_signal = state.first_sync_done;
        let mut room_list_state_signal = state.room_list_state;
        let mut rooms_list_signal = state.rooms_list;
        let mut is_rooms_loading_signal = state.is_rooms_loading;
        let mut space_list_signal = state.space_list;
        let mut is_spaces_loading_signal = state.is_spaces_loading;
        let mut space_rooms_map_signal = state.space_rooms_map;

        spawn(async move {
            while let Ok(event) = rx.recv().await {
                println!("DEBUG: AppState received event: {:?}", event);
                match event {
                    MatrixEvent::SyncStateChanged(sync_state) => {
                        match sync_state {
                            SyncState::Running => {
                                is_syncing_signal.set(true);
                            }
                            SyncState::Terminated => {
                                is_syncing_signal.set(false);
                            }
                            SyncState::Error(e) => {
                                eprintln!("Sync service error: {:?}", e);
                            }
                            _ => {}
                        }
                    }
                    MatrixEvent::RoomListStateChanged(rl_state) => {
                        room_list_state_signal.set(rl_state.clone());
                        if matches!(rl_state, RoomListState::Running) {
                            first_sync_done_signal.set(true);
                        }
                    }
                    MatrixEvent::LoggedOut => {
                        is_syncing_signal.set(false);
                        first_sync_done_signal.set(false);
                        room_list_state_signal.set(RoomListState::Init);
                        rooms_list_signal.set(Vec::new());
                        is_rooms_loading_signal.set(true);
                        space_list_signal.set(Vec::new());
                        is_spaces_loading_signal.set(true);
                        space_rooms_map_signal.set(HashMap::new());
                    }
                    _ => {}
                }
            }
        });

        state
    }
}
