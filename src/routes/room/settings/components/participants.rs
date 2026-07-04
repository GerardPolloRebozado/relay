use dioxus::prelude::*;
use matrix_sdk::{
    media::MediaFormat,
    room::RoomMemberRole,
    ruma::{OwnedRoomId, events::room::power_levels::UserPowerLevel},
};

use crate::{
    components::{
        avatar::{AvatarImageSize, ImageAvatar},
        badge::{Badge, BadgeVariant},
        card::{Card, CardContent},
    },
    routes::{home::dm_utilities::get_room_avatar, router::Route},
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
            CardContent {
            class: Styles::user_card,
            ImageAvatar {
                src: user.avatar_url,
                size: AvatarImageSize::Large,
            }
            div {
                p {
                    {user.name}
                }
                p {
                    {user.id}
                }
            }
            div {
                class: Styles::badge_container,
                {
                    match user.role {
                        RoomMemberRole::Creator => rsx!{
                            Badge {
                                {"Creator"}
                            }
                        },
                        RoomMemberRole::Administrator => rsx!{
                            Badge{
                                {"Administrator"}
                            }
                        },
                        RoomMemberRole::Moderator => rsx!{
                            Badge{
                                {"Moderator"}
                            }
                        },
                        RoomMemberRole::User => rsx!{
                            Badge{
                                variant: BadgeVariant::Secondary,
                                {"User"}
                            }
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

    use_future(move || {
        let cloned_id = id.clone();
        async move {
            let state = use_context::<AppState>();
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
                for user_id in joined_user_ids {
                    let profile = room.get_member(&user_id).await;
                    if profile.is_err() {
                        error!(" Could not get profile information {}", user_id);
                        continue;
                    }
                    let profile = profile.unwrap();
                    if profile.is_none() {
                        error!("Could not find profile {}", user_id);
                        continue;
                    }
                    let profile = profile.unwrap();

                    let mut avatar_url = String::new();
                    if let Ok(bytes) = profile.avatar(MediaFormat::File).await
                        && let Some(bytes_unwrapped) = bytes
                    {
                        avatar_url = encode_to_data_uri(bytes_unwrapped).unwrap_or("".to_string());
                    }
                    user_list.write().push(BasicUserInfo {
                        id: user_id.to_string(),
                        name: profile.name().to_string(),
                        avatar_url,
                        role: profile.suggested_role_for_power_level(),
                    });
                }
            }
        }
    });

    rsx! {
        div {
            class: Styles::list,
            for user in user_list.read().iter() {
                ParticipantCard { user: user.clone() },
            }
        }
    }
}
