use dioxus::prelude::*;
use matrix_sdk::{media::MediaFormat, room::RoomMemberRole, ruma::OwnedRoomId};

use crate::{
    components::{
        avatar::{AvatarImageSize, ImageAvatar},
        badge::{Badge, BadgeVariant},
        card::{Card, CardContent},
    },
    routes::router::Route,
    state::app_state::AppState,
    utilities::media::encode_to_data_uri,
};

#[css_module("src/routes/room/settings/components/participants.css")]
struct Styles;

#[derive(Clone, PartialEq)]
struct BasicUserInfo {
    pub id: String,
    pub name: String,
    pub avatar_url: String,
    pub role: RoomMemberRole,
}

#[component]
fn ParticipantCard(user: BasicUserInfo) -> Element {
    rsx! {
        Card {
            CardContent { class: Styles::user_card,
                ImageAvatar { src: user.avatar_url, size: AvatarImageSize::Large }
                div {
                    p { {user.name} }
                    p { {user.id} }
                }
                div { class: Styles::badge_container,
                    {
                        match user.role {
                            RoomMemberRole::Creator => rsx! {
                                Badge { {"Creator"} }
                            },
                            RoomMemberRole::Administrator => rsx! {
                                Badge { {"Administrator"} }
                            },
                            RoomMemberRole::Moderator => rsx! {
                                Badge { {"Moderator"} }
                            },
                            RoomMemberRole::User => rsx! {
                                Badge { variant: BadgeVariant::Secondary, {"User"} }
                            },
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ParticipantsList(id: OwnedRoomId) -> Element {
    let mut user_list = use_signal(Vec::<BasicUserInfo>::new);
    let state = use_context::<AppState>();

    use_future(move || {
        let cloned_id = id.clone();
        async move {
            let matrix_manager = state.matrix.read().clone();
            let client = matrix_manager.client().await.unwrap();
            let room = client.get_room(&cloned_id);
            if room.is_none() {
                error!("Could not get room {}", cloned_id);
                navigator().push(Route::Home);
                return;
            }
            let room = room.unwrap();

            if let Ok(joined_user_ids) = room.joined_user_ids().await {
                let mut temp_list = Vec::new();
                for user_id in joined_user_ids {
                    let profile = room.get_member(&user_id).await;
                    match profile {
                        Ok(Some(profile)) => {
                            let mut avatar_url = String::new();
                            if let Ok(Some(bytes)) = profile.avatar(MediaFormat::File).await {
                                avatar_url = encode_to_data_uri(bytes).unwrap_or_default();
                            }
                            temp_list.push(BasicUserInfo {
                                id: user_id.to_string(),
                                name: profile.name().to_string(),
                                avatar_url,
                                role: profile.suggested_role_for_power_level(),
                            });
                        }
                        Ok(None) => {
                            error!("Could not find profile {}", user_id);
                        }
                        Err(e) => {
                            error!("Could not get profile information {}: {:?}", user_id, e);
                        }
                    }
                }
                user_list.set(temp_list);
            }
        }
    });

    rsx! {
        div { class: Styles::list,
            {format!("{} participants", user_list.read().len())}
            for user in user_list.read().iter() {
                ParticipantCard { user: user.clone() }
            }
        }
    }
}
