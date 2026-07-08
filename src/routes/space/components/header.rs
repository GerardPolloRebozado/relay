use crate::components::avatar::{AvatarImageSize, AvatarShape, ImageAvatar};
use crate::components::go_back::GoBackButton;
use crate::components::header::Header;
use crate::components::spinner::Spinner;
use crate::routes::home::dm_utilities::get_room_avatar;
use crate::routes::router::Route;
use crate::state::app_state::AppState;
use dioxus::prelude::*;
use matrix_sdk::ruma::room::RoomType;

use crate::custom_types::rooms::RoomContainer;

#[css_module("src/routes/space/components/header.css")]
struct Styles;

#[component]
pub fn SpaceHeader(space: RoomContainer) -> Element {
    let state = use_context::<AppState>();
    let cloned_space = space.clone();
    let mut memo_space = use_signal(|| cloned_space.clone());
    if *memo_space.read() != space {
        memo_space.set(space.clone());
    }

    let space_info = use_resource(move || {
        let _cloned_space = memo_space.read().clone();
        let matrix_manager = state.matrix.read().clone();
        async move {
            let client = matrix_manager.client().await.unwrap();
            let display_name = _cloned_space.0.display_name().await;
            let name = match display_name {
                Ok(dn) => dn.to_string(),
                Err(_) => "Unknown Space".to_string(),
            };
            let avatar_url = get_room_avatar(&client, &_cloned_space.0)
                .await
                .unwrap_or(String::new());
            (name, avatar_url)
        }
    });

    if space.0.room_type().is_none() || space.0.room_type().unwrap() != RoomType::Space {
        navigator().push(Route::Home);
    }

    match &*space_info.read_unchecked() {
        Some((name, avatar_url)) => rsx! {
            Header {
                div {
                    class: Styles::name_image,
                    GoBackButton{},
                    ImageAvatar {
                        size: AvatarImageSize::Medium,
                        shape: AvatarShape::Rounded,
                        src: avatar_url,
                        {name.clone()},
                    }
                    h2 {
                        {name.clone()}
                    }
                }
            }
        },
        None => rsx! { Spinner {} },
    }
}
