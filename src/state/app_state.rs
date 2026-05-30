use crate::services::matrix::client_manager::{MatrixEvent, MatrixManager};
use dioxus::prelude::*;
use matrix_sdk_ui::room_list_service::State as RoomListState;
use matrix_sdk_ui::sync_service::State as SyncState;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Copy)]
pub struct AppState {
    pub matrix: Signal<Arc<RwLock<MatrixManager>>>,
    pub is_syncing: Signal<bool>,
    pub is_loaded: Signal<bool>,
    pub first_sync_done: Signal<bool>,
    pub room_list_state: Signal<RoomListState>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let (manager, mut rx) = MatrixManager::new();
        let manager = Arc::new(RwLock::new(manager));

        let state = Self {
            matrix: Signal::new(manager.clone()),
            is_syncing: Signal::new(false),
            is_loaded: Signal::new(false),
            first_sync_done: Signal::new(false),
            room_list_state: Signal::new(RoomListState::Init),
        };

        let mut is_syncing_signal = state.is_syncing;
        let mut first_sync_done_signal = state.first_sync_done;
        let mut room_list_state_signal = state.room_list_state;

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
                    }
                    _ => {}
                }
            }
        });

        state
    }
}
