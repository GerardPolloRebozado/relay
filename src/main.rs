#![allow(non_snake_case)]
pub mod components;
pub mod custom_types;
pub mod layouts;
pub mod routes;
pub mod services;
pub mod state;
pub mod utilities;

use crate::routes::router::Route;
use crate::state::secure_state::init_secure_storage;
#[cfg(feature = "desktop")]
use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;
use log::debug;
use state::app_state::AppState;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

type CacheEntry = (Arc<Vec<u8>>, String);
type CacheMap = HashMap<String, CacheEntry>;

pub static HTTP_PORT: OnceLock<u16> = OnceLock::new();
pub static MEDIA_CACHE: OnceLock<RwLock<CacheMap>> = OnceLock::new();

fn main() {
    if std::env::var("GST_PLUGIN_SYSTEM_PATH_1_0").is_err()
        && let Some(path) = option_env!("GST_PLUGIN_SYSTEM_PATH_1_0")
    {
        unsafe {
            std::env::set_var("GST_PLUGIN_SYSTEM_PATH_1_0", path);
        }
    }

    env_logger::init();
    init_secure_storage();

    MEDIA_CACHE.get_or_init(|| RwLock::new(HashMap::new()));

    std::thread::spawn(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let port = start_http_server().await;
            let _ = HTTP_PORT.set(port);
            std::future::pending::<()>().await;
        });
    });

    while HTTP_PORT.get().is_none() {
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    let builder = dioxus::LaunchBuilder::new();

    #[cfg(feature = "desktop")]
    let builder = builder.with_cfg(
        Config::default().with_menu(None).with_window(
            WindowBuilder::new()
                .with_maximized(true)
                .with_title("Relay Messaging"),
        ),
    );

    builder.launch(App);
}

fn App() -> Element {
    let mut state = use_context_provider(AppState::new);

    use_context_provider(state::notifications::NotificationsState::default);

    let dark_mode = use_signal(|| true);

    use_future(move || async move {
        let manager = state.matrix.cloned();
        if manager.load_from_storage().await.is_some() {
            debug!("Loaded client from storage");
            let _ = manager.start_sync().await;
        }
        state.is_loaded.set(true);
    });

    rsx! {
        Stylesheet { href: asset!("/assets/main.css") }
        div { class: if dark_mode() { "dark" } else { "" },
            div { id: "app-container",
                if !state.is_loaded.cloned() {
                    div { class: "loader-container",
                        div { class: "loader" }
                    }
                } else {
                    layouts::notifications::Notifications {}
                    Router::<Route> {}
                }
            }
        }
    }
}

fn parse_range(range_str: &str, file_len: u64) -> Option<(u64, u64)> {
    if !range_str.starts_with("bytes=") {
        return None;
    }
    let range = &range_str["bytes=".len()..];
    let parts: Vec<&str> = range.split('-').collect();
    if parts.is_empty() {
        return None;
    }
    let start = parts[0].parse::<u64>().ok().unwrap_or(0);
    let end = if parts.len() > 1 && !parts[1].is_empty() {
        parts[1].parse::<u64>().ok().unwrap_or(file_len - 1)
    } else {
        file_len - 1
    };
    let end = std::cmp::min(end, file_len - 1);
    if start > end {
        None
    } else {
        Some((start, end))
    }
}

async fn start_http_server() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut buf = [0u8; 4096];
                if let Ok(n) = socket.read(&mut buf).await {
                    let req = String::from_utf8_lossy(&buf[..n]);
                    let lines: Vec<&str> = req.lines().collect();
                    if lines.is_empty() {
                        return;
                    }
                    let parts: Vec<&str> = lines[0].split_whitespace().collect();
                    if parts.len() < 2 || parts[0] != "GET" {
                        return;
                    }

                    let key = parts[1]
                        .split('?')
                        .next()
                        .unwrap_or(parts[1])
                        .trim_start_matches('/');

                    let mut range_header = None;
                    for line in &lines {
                        if line.to_lowercase().starts_with("range:") {
                            range_header = Some(line["range:".len()..].trim().to_string());
                        }
                    }

                    let entry = {
                        let cache = MEDIA_CACHE.get().unwrap().read().unwrap();
                        cache.get(key).cloned()
                    };

                    let (data, mime_type) = match entry {
                        Some(e) => e,
                        None => {
                            let _ = socket.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").await;
                            return;
                        }
                    };

                    let file_len = data.len() as u64;
                    let (start, end) = if let Some(range_str) = range_header.as_ref() {
                        parse_range(range_str, file_len).unwrap_or((0, file_len - 1))
                    } else {
                        (0, file_len - 1)
                    };

                    let slice = &data[start as usize..=end as usize];
                    let is_range = range_header.is_some();
                    let status_line = if is_range {
                        "HTTP/1.1 206 Partial Content"
                    } else {
                        "HTTP/1.1 200 OK"
                    };

                    let mut response = format!(
                        "{}\r\n\
                        Content-Type: {}\r\n\
                        Accept-Ranges: bytes\r\n\
                        Access-Control-Allow-Origin: *\r\n\
                        Connection: close\r\n\
                        Content-Length: {}\r\n",
                        status_line,
                        mime_type,
                        slice.len()
                    );

                    if is_range {
                        response.push_str(&format!(
                            "Content-Range: bytes {}-{}/{}\r\n",
                            start, end, file_len
                        ));
                    }
                    response.push_str("\r\n");

                    let _ = socket.write_all(response.as_bytes()).await;
                    let _ = socket.write_all(slice).await;
                }
            });
        }
    });

    port
}
