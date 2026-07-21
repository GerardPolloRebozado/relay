use dioxus::prelude::*;
use matrix_sdk::ruma::OwnedUserId;

use crate::{
    components::avatar::{AvatarImageSize, ImageAvatar},
    state::app_state::AppState,
    utilities::media::{AvatarSize, get_user_avatar},
};

#[component]
pub fn ReadReceipt(user_id: OwnedUserId) -> Element {
    let state = use_context::<AppState>();

    let avatar_resource = use_resource(move || {
        let user_id = user_id.clone();
        let state = state;
        async move {
            let matrix = state.matrix.read().clone();
            let client = matrix.client().await?;
            get_user_avatar(&client, &user_id, AvatarSize::Small).await
        }
    });

    let avatar_url = avatar_resource
        .read()
        .as_ref()
        .and_then(|opt| opt.clone())
        .unwrap_or_default();

    rsx! {
        ImageAvatar { src: "{avatar_url}", size: AvatarImageSize::Small }
    }
}
