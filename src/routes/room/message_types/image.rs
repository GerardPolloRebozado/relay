use crate::{state::app_state::AppState, utilities::media::get_media_data_uri};
use dioxus::prelude::*;
use matrix_sdk::ruma::events::room::message::ImageMessageEventContent;

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

    let image_resource = use_resource(move || {
        let source = img_clone.source.clone();

        async move {
            let state = use_context::<AppState>();
            let matrix = state.matrix.cloned();

            if let Some(client) = matrix.client().await {
                return get_media_data_uri(&client, source).await;
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
