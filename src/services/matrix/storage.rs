use keyring_core::Entry;
use matrix_sdk::authentication::matrix::MatrixSession;
use matrix_sdk::config::StoreConfig;
use matrix_sdk::cross_process_lock::CrossProcessLockConfig;
use matrix_sdk::{AuthSession, Client, ClientBuilder};
use matrix_sdk_sqlite::{
    SqliteCryptoStore, SqliteEventCacheStore, SqliteMediaStore, SqliteStateStore,
};
use rand::{TryRngCore, rngs::OsRng};
use std::path::PathBuf;

fn get_matrix_storage_dir() -> PathBuf {
    #[cfg(target_os = "android")]
    {
        PathBuf::from("/data/data/dev.gerarddupre.relay/files/relay")
    }
    #[cfg(not(target_os = "android"))]
    {
        let mut path = sysdirs::data_dir().expect("Could not determine system data directory");

        #[cfg(not(target_os = "ios"))]
        {
            path.push("relay");
        }

        path.push("relay");

        path
    }
}

const KEYCHAIN_SERVICE: &str = "com.relay.app";
const KEYCHAIN_ACCOUNT: &str = "matrix_db_passphrase";
const KEYCHAIN_HOMESERVER: &str = "matrix_homeserver_url";
const KEYCHAIN_SESSION: &str = "matrix_user_session";

pub async fn load_client_from_storage() -> Option<Client> {
    let entry = Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT).ok()?;
    let token = entry.get_password().ok()?;

    let hs_entry = Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_HOMESERVER).ok()?;
    let homeserver_url = hs_entry.get_password().ok()?;

    let storage_dir = get_matrix_storage_dir();
    let passphrase = Some(token.as_str());

    let state_store = SqliteStateStore::open(storage_dir.clone(), passphrase)
        .await
        .ok()?;
    let crypto_store = SqliteCryptoStore::open(storage_dir.clone(), passphrase)
        .await
        .ok()?;
    let event_cache_store = SqliteEventCacheStore::open(storage_dir.clone(), passphrase)
        .await
        .ok()?;
    let media_store = SqliteMediaStore::open(storage_dir.clone(), passphrase)
        .await
        .ok()?;

    let store_config = StoreConfig::new(CrossProcessLockConfig::multi_process("relay"))
        .crypto_store(crypto_store)
        .state_store(state_store)
        .event_cache_store(event_cache_store)
        .media_store(media_store);

    let client = Client::builder()
        .with_encryption_settings(matrix_sdk::encryption::EncryptionSettings {
            auto_enable_cross_signing: true,
            backup_download_strategy:
                matrix_sdk::encryption::BackupDownloadStrategy::AfterDecryptionFailure,
            auto_enable_backups: true,
        })
        .homeserver_url(homeserver_url)
        .store_config(store_config)
        .build()
        .await
        .ok()?;

    let session_entry = Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_SESSION).ok()?;
    if let Ok(serialized_session) = session_entry.get_password()
        && let Ok(session) = serde_json::from_str::<MatrixSession>(&serialized_session)
        && client.restore_session(session).await.is_ok()
    {
        client
            .encryption()
            .wait_for_e2ee_initialization_tasks()
            .await;
        return Some(client);
    }

    None
}

pub async fn clear_storage() -> Result<(), String> {
    let entry = Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT);
    if let Ok(e) = entry {
        let _ = e.delete_credential();
    }

    let hs_entry = Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_HOMESERVER);
    if let Ok(e) = hs_entry {
        let _ = e.delete_credential();
    }

    let session_entry = Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_SESSION);
    if let Ok(e) = session_entry {
        let _ = e.delete_credential();
    }

    Ok(())
}

pub async fn setup_client_builder(
    mut client_builder: ClientBuilder,
) -> Result<ClientBuilder, String> {
    let entry = Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT)
        .map_err(|e| format!("Failed to access native keychain: {}", e))?;

    let passphrase = match entry.get_password() {
        Ok(pw) => pw,
        Err(_) => {
            let mut key = [0u8; 32];
            if OsRng.try_fill_bytes(&mut key).is_err() {
                return Err("Error generating number".to_string());
            }
            let pw = hex::encode(key);
            entry
                .set_password(&pw)
                .map_err(|e| format!("Failed to save passphrase to keychain: {}", e))?;
            pw
        }
    };

    let storage_dir = get_matrix_storage_dir();

    let state_store = SqliteStateStore::open(storage_dir.clone(), Some(passphrase.as_str()))
        .await
        .map_err(|e| format!("Error opening state store: {}", e))?;

    let crypto_store = SqliteCryptoStore::open(storage_dir.clone(), Some(passphrase.as_str()))
        .await
        .map_err(|e| format!("Error opening crypto store: {}", e))?;

    let event_cache_store =
        SqliteEventCacheStore::open(storage_dir.clone(), Some(passphrase.as_str()))
            .await
            .map_err(|e| format!("Error opening event store: {}", e))?;

    let media_store = SqliteMediaStore::open(storage_dir.clone(), Some(passphrase.as_str()))
        .await
        .map_err(|e| format!("Error opening media store: {}", e))?;

    let store_config = StoreConfig::new(CrossProcessLockConfig::multi_process("relay"))
        .crypto_store(crypto_store)
        .state_store(state_store)
        .event_cache_store(event_cache_store)
        .media_store(media_store);

    // TODO: user should be able to enable disable backups
    client_builder = client_builder
        .store_config(store_config)
        .with_encryption_settings(matrix_sdk::encryption::EncryptionSettings {
            auto_enable_cross_signing: true,
            backup_download_strategy:
                matrix_sdk::encryption::BackupDownloadStrategy::AfterDecryptionFailure,
            auto_enable_backups: true,
        });

    Ok(client_builder)
}

pub fn save_homeserver_url(url: &str) -> Result<(), String> {
    let entry = Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_HOMESERVER)
        .map_err(|e| format!("Failed to access native keychain: {}", e))?;
    entry
        .set_password(url)
        .map_err(|e| format!("Failed to save homeserver URL to keychain: {}", e))?;
    Ok(())
}

pub fn save_matrix_session(client: &Client) -> Result<(), String> {
    if let Some(AuthSession::Matrix(session)) = client.session() {
        let serialized = serde_json::to_string(&session)
            .map_err(|e| format!("Failed to serialize session: {}", e))?;

        let entry = Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_SESSION)
            .map_err(|e| format!("Failed to access native keychain: {}", e))?;

        entry
            .set_password(&serialized)
            .map_err(|e| format!("Failed to save session to keychain: {}", e))?;

        Ok(())
    } else {
        Err("No Matrix session available to save".to_string())
    }
}

pub fn delete_database_files() -> std::io::Result<()> {
    let dir = get_matrix_storage_dir();
    std::fs::remove_dir_all(dir)
}
