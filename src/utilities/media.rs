use crate::state::app_state::AppState;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use dioxus::{hooks::use_context, signals::ReadableExt};
use log::error;
use matrix_sdk::{
    media::{MediaFormat, MediaRequestParameters, MediaThumbnailSettings},
    ruma::{OwnedMxcUri, events::room::MediaSource, media::Method},
};

pub async fn get_img(mxc: OwnedMxcUri) -> Option<String> {
    let format = MediaFormat::Thumbnail(MediaThumbnailSettings {
        method: Method::Crop,
        width: 50u32.into(),
        height: 50u32.into(),
        animated: false,
    });
    let app_state = use_context::<AppState>();
    let client = app_state.matrix.cloned().client().await?;
    if let Ok(bytes) = client
        .media()
        .get_media_content(
            &MediaRequestParameters {
                source: MediaSource::Plain(mxc),
                format,
            },
            true,
        )
        .await
    {
        return encode_to_data_uri(bytes);
    }
    None
}

pub fn encode_to_data_uri(bytes: Vec<u8>) -> Option<String> {
    let b64_string = STANDARD.encode(&bytes);
    let mime_type = infer::get(&bytes);
    if mime_type.is_none() {
        error!("Could not determine MIME type for bytes");
        return None;
    }
    let mime_type = mime_type.unwrap().mime_type();
    Some(format!("data:{};base64,{}", mime_type, b64_string))
}
