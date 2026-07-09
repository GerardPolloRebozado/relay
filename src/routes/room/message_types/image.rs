use crate::state::app_state::AppState;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use dioxus::prelude::*;
use matrix_sdk::{
    media::{MediaFormat, MediaRequestParameters},
    ruma::events::room::message::ImageMessageEventContent,
};

#[css_module("/src/routes/room/message_types/image.css")]
struct Styles;

#[derive(Clone)]
pub struct ImagePayload(pub ImageMessageEventContent);

impl PartialEq for ImagePayload {
    fn eq(&self, other: &Self) -> bool {
        self.0.body == other.0.body
    }
}

#[component]
pub fn ImageMessage(payload: ImagePayload) -> Element {
    let img = payload.0;
    let img_clone = img.clone();

    let mime_type = img
        .info
        .as_ref()
        .and_then(|info| info.mimetype.clone())
        .unwrap_or_else(|| "image/png".to_string());

    let image_resource = use_resource(move || {
        let source = img_clone.source.clone();
        let mime = mime_type.clone();

        async move {
            let state = use_context::<AppState>();
            let matrix = state.matrix.cloned();

            if let Some(client) = matrix.client().await {
                let request = MediaRequestParameters {
                    source,
                    format: MediaFormat::File,
                };

                if let Ok(img_content) = client.media().get_media_content(&request, true).await {
                    let base64_string = STANDARD.encode(img_content);
                    return Some(format!("data:{};base64,{}", mime, base64_string));
                }
            }
            None
        }
    });

    match &*image_resource.read_unchecked() {
        Some(Some(base64_src)) => rsx! {
            img { src: "{base64_src}", alt: "{img.body}", class: Styles::img }
        },
        Some(None) => rsx! {
            span { style: "color: red; font-style: italic;", "[Failed to load image: {img.body}]" }
        },
        None => rsx! {
            span { style: "color: gray; font-style: italic;", "[Loading image: {img.body}...]" }
        },
    }
}
