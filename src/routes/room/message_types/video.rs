use crate::state::app_state::AppState;
use dioxus::prelude::*;
use matrix_sdk::{
    media::{MediaFormat, MediaRequestParameters},
    ruma::{events::room::message::VideoMessageEventContent, events::room::MediaSource},
};
use std::sync::Arc;

#[css_module("/src/routes/room/message_types/video.css")]
struct Styles;

#[derive(Clone)]
pub struct VideoPayload(pub VideoMessageEventContent);

impl PartialEq for VideoPayload {
    fn eq(&self, other: &Self) -> bool {
        self.0.body == other.0.body
    }
}

#[derive(Clone, PartialEq)]
enum PlayState {
    Idle,
    Loading,
    Ready(String),
    Failed,
}

fn media_cache_key(video: &VideoMessageEventContent) -> String {
    let mxc_uri = match &video.source {
        MediaSource::Plain(mxc) => mxc.to_string(),
        MediaSource::Encrypted(file) => file.url.to_string(),
    };
    mxc_uri
        .replace("mxc://", "")
        .replace("/", "_")
        .replace(":", "_")
}

#[component]
pub fn VideoMessage(payload: VideoPayload) -> Element {
    let video = payload.0;
    let mut play_state = use_signal(|| PlayState::Idle);
    let matrix = use_context::<AppState>().matrix.cloned();

    let mime_type = video
        .info
        .as_ref()
        .and_then(|info| info.mimetype.clone())
        .unwrap_or_else(|| "video/mp4".to_string());

    let duration_text = video
        .info
        .as_ref()
        .and_then(|info| info.duration)
        .map(|d| {
            let secs = d.as_secs();
            format!("{}:{:02}", secs / 60, secs % 60)
        });

    let aspect_style = video
        .info
        .as_ref()
        .and_then(|info| {
            let w = info.width.as_ref()?;
            let h = info.height.as_ref()?;
            Some(format!("aspect-ratio: {} / {};", w, h))
        })
        .unwrap_or_else(|| "aspect-ratio: 16 / 9;".to_string());

    let on_play = {
        let video = video.clone();
        let mime = mime_type.clone();
        let matrix = matrix.clone();
        move |_| {
            if *play_state.read() != PlayState::Idle {
                return;
            }
            play_state.set(PlayState::Loading);

            let video = video.clone();
            let mime = mime.clone();
            let matrix = matrix.clone();

            spawn(async move {
                let key = media_cache_key(&video);
                let port = *crate::HTTP_PORT.get().unwrap_or(&8080);
                let url = format!("http://127.0.0.1:{}/{}", port, key);

                // Check in-memory cache first
                {
                    let cache = crate::MEDIA_CACHE.get().unwrap().read().unwrap();
                    if cache.contains_key(&key) {
                        play_state.set(PlayState::Ready(url));
                        return;
                    }
                }

                // Download and decrypt from Matrix server
                if let Some(client) = matrix.client().await {
                    let request = MediaRequestParameters {
                        source: video.source.clone(),
                        format: MediaFormat::File,
                    };
                    if let Ok(content) =
                        client.media().get_media_content(&request, true).await
                    {
                        {
                            let mut cache =
                                crate::MEDIA_CACHE.get().unwrap().write().unwrap();
                            cache.insert(key, (Arc::new(content), mime));
                        }
                        play_state.set(PlayState::Ready(url));
                        return;
                    }
                }
                play_state.set(PlayState::Failed);
            });
        }
    };

    match &*play_state.read() {
        PlayState::Idle => rsx! {
            div {
                class: Styles::play_overlay,
                style: "{aspect_style}",
                onclick: on_play,
                div { class: Styles::play_button,
                    div { class: Styles::play_icon }
                }
                if let Some(dur) = &duration_text {
                    span { class: Styles::duration, "{dur}" }
                }
            }
        },
        PlayState::Loading => rsx! {
            div { class: Styles::play_overlay, style: "{aspect_style}",
                div { class: Styles::spinner }
            }
        },
        PlayState::Ready(src) => rsx! {
            video {
                controls: true,
                autoplay: true,
                src: "{src}",
                class: Styles::video,
            }
        },
        PlayState::Failed => rsx! {
            div { class: Styles::play_overlay, style: "{aspect_style}",
                span { class: Styles::error_text, "Failed to load video" }
            }
        },
    }
}
