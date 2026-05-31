use crate::services::matrix::storage::{
    clear_storage, load_client_from_storage, save_homeserver_url, save_matrix_session,
    setup_client_builder,
};
use matrix_sdk::ruma::UserId;
use matrix_sdk::Client;
use matrix_sdk_ui::room_list_service::{RoomListService, State as RoomListState};
use matrix_sdk_ui::sync_service::{State as SyncState, SyncService};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

#[derive(Clone, Debug)]
pub enum MatrixEvent {
    SyncStateChanged(SyncState),
    RoomListStateChanged(RoomListState),
    ClientLoaded(Client),
    LoggedOut,
}

#[derive(Clone)]
pub struct MatrixManager {
    inner: Arc<RwLock<MatrixManagerInner>>,
    event_tx: broadcast::Sender<MatrixEvent>,
}

#[derive(Default)]
struct MatrixManagerInner {
    client: Option<Client>,
    sync_service: Option<Arc<SyncService>>,
    room_list_service: Option<Arc<RoomListService>>,
}

impl MatrixManager {
    pub fn new() -> (Self, broadcast::Receiver<MatrixEvent>) {
        let (tx, rx) = broadcast::channel(100);
        (
            Self {
                inner: Arc::new(RwLock::new(MatrixManagerInner {
                    client: None,
                    sync_service: None,
                    room_list_service: None,
                })),
                event_tx: tx,
            },
            rx,
        )
    }

    pub async fn load_from_storage(&self) -> Option<Client> {
        if let Some(client) = load_client_from_storage().await {
            let mut inner = self.inner.write().await;
            inner.client = Some(client.clone());
            let _ = self.event_tx.send(MatrixEvent::ClientLoaded(client.clone()));
            Some(client)
        } else {
            None
        }
    }

    pub async fn login(
        &self,
        user_id: &UserId,
        password: &str,
    ) -> Result<Client, String> {
        let client_builder = setup_client_builder(Client::builder().server_name(user_id.server_name()))
            .await?;
        
        let client = client_builder.build().await.map_err(|e| e.to_string())?;
        
        client
            .matrix_auth()
            .login_username(user_id, password)
            .initial_device_display_name("relay")
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let _ = save_homeserver_url(client.homeserver().as_str());
        let _ = save_matrix_session(&client);

        {
            let mut inner = self.inner.write().await;
            inner.client = Some(client.clone());
        }
        let _ = self.event_tx.send(MatrixEvent::ClientLoaded(client.clone()));
        
        Ok(client)
    }

    pub async fn logout(&self) {
        let (sync_service, client) = {
            let mut inner = self.inner.write().await;
            let sync_service = inner.sync_service.take();
            let client = inner.client.take();
            inner.room_list_service = None;
            (sync_service, client)
        };

        if let Some(sync_service) = sync_service {
            let _ = sync_service.stop();
        }

        if let Some(client) = client {
            let _ = client.logout().await;
        }

        let _ = clear_storage().await;
        let _ = self.event_tx.send(MatrixEvent::LoggedOut);
    }

    pub async fn start_sync(&self) -> Result<(), String> {
        let client = {
            let inner = self.inner.read().await;
            let Some(client) = inner.client.as_ref().cloned() else {
                return Err("No client available to sync".to_string());
            };

            if inner.sync_service.is_some() {
                return Ok(());
            }
            client
        };

        println!("DEBUG: Starting SyncService...");
        
        // Ensure event cache is subscribed to
        let _ = client.event_cache().subscribe().map_err(|e| e.to_string())?;

        let sync_service = Arc::new(
            SyncService::builder(client.clone())
                .build()
                .await
                .map_err(|e| e.to_string())?,
        );
        
        let room_list_service = sync_service.room_list_service();
        
        {
            let mut inner = self.inner.write().await;
            inner.room_list_service = Some(room_list_service.clone());
            inner.sync_service = Some(sync_service.clone());
        }

        let event_tx = self.event_tx.clone();
        let mut state_stream = sync_service.state();
        
        tokio::spawn(async move {
            while let Some(state) = state_stream.next().await {
                println!("DEBUG: SyncService state: {:?}", state);
                let _ = event_tx.send(MatrixEvent::SyncStateChanged(state.clone()));
                if matches!(state, SyncState::Terminated) {
                    break;
                }
            }
        });

        let event_tx_rl = self.event_tx.clone();
        let mut rl_state_stream = room_list_service.state();
        tokio::spawn(async move {
            while let Some(state) = rl_state_stream.next().await {
                println!("DEBUG: RoomListService state: {:?}", state);
                let _ = event_tx_rl.send(MatrixEvent::RoomListStateChanged(state.clone()));
            }
        });

        sync_service.start().await;
        println!("DEBUG: SyncService.start() called");

        Ok(())
    }

    pub async fn client(&self) -> Option<Client> {
        self.inner.read().await.client.clone()
    }

    pub async fn sync_service(&self) -> Option<Arc<SyncService>> {
        self.inner.read().await.sync_service.clone()
    }

    pub async fn room_list_service(&self) -> Option<Arc<RoomListService>> {
        self.inner.read().await.room_list_service.clone()
    }
}
