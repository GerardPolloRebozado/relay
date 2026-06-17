use crate::{
    components::{
        avatar::{AvatarImageSize, ImageAvatar},
        badge::{Badge, BadgeVariant},
        button::Button,
        card::{Card, CardContent, CardDescription, CardHeader, CardTitle},
        dialog::{Dialog, DialogTitle},
        dropdown_menu::{DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger},
        input::Input,
        item::{Item, ItemVariant},
    },
    routes::router::Route,
    state::app_state::AppState,
    utilities::media::get_img,
};
use dioxus::prelude::*;
use dioxus_icons::lucide::Plus;
use dioxus_icons::lucide::User;
use dioxus_router::components::Link;
use matrix_sdk::ruma::{OwnedRoomId, OwnedUserId, UserId};
use matrix_sdk::{ruma::api::client::room::create_room::v3::Request as CreateRoomRequest, Client};
use std::collections::HashMap;

#[derive(Clone, PartialEq)]
pub struct DMInfo {
    pub room_id: OwnedRoomId,
    pub name: String,
    pub avatar_url: String,
    pub last_message: String,
}

#[css_module("/src/routes/home/components.css")]
struct Styles;

#[component]
pub fn DMCard(dm: DMInfo) -> Element {
    rsx! {
        Link {
            to: Route::Room {
                id: dm.room_id.clone(),
            },
            class: "no-link-highlight",
            Card { key: "{dm.room_id}",
                CardHeader { class: Styles::card_header,
                    ImageAvatar {
                        src: "{dm.avatar_url}",
                        alt: "{dm.name}",
                        size: AvatarImageSize::Medium,
                        "{dm.name.chars().next().unwrap_or('?')}"
                    }
                    CardTitle { "{dm.name}" }
                }
                CardContent {
                    CardDescription { "{dm.last_message}" }
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct UserSearchResult {
    user_id: String,
    avatar_url: Option<String>,
}

#[component]
pub fn NewRoomModal(mut open: Signal<bool>) -> Element {
    let mut search_term = use_signal(|| "".to_string());
    let mut search_results = use_signal(|| Vec::<UserSearchResult>::new());
    let mut selected_users = use_signal(|| Vec::<UserSearchResult>::new());
    let app_state = use_context::<AppState>();

    use_effect(move || {
        let value = search_term.cloned();
        spawn(async move {
            let client = app_state.matrix.cloned().client().await;
            if client.is_none() {
                error!("Error getting client");
                return;
            }
            let client = client.unwrap();
            let response = client.search_users(&value, 10).await;
            if response.is_err() {
                error!("Error searching users");
                return;
            }
            let response = response.unwrap();
            let mut new_results = Vec::new();
            for user in response.results {
                let user_id = user.user_id.to_string();
                let avatar_url = if let Some(url) = user.avatar_url {
                    get_img(url).await
                } else {
                    None
                };
                new_results.push(UserSearchResult {
                    user_id,
                    avatar_url,
                });
            }
            search_results.set(new_results);
        });
    });

    rsx! {
        Dialog {
            class: Styles::new_dm_dialog,
            open: open(),
            on_open_change: move |v| open.set(v),
            DialogTitle { "Create Direct Message" }
            {
                if selected_users.len() > 0 {
                    rsx! {
                        div {
                            p{
                                {"Selected users: ".to_string()},
                            }
                            div {
                            class: Styles::selected_users_list,
                            for (i, user) in selected_users.iter().enumerate() {
                                    Item {
                                        class: Styles::selected_user,
                                        onclick: move |_| {
                                            selected_users.write().remove(i);
                                        },
                                        variant: ItemVariant::Outline,

                                        {
                                            let avatar_url = user.avatar_url.clone().unwrap_or_default();
                                            rsx! {
                                                ImageAvatar {
                                                    src: "{avatar_url}",
                                                    alt: "User profile picture",
                                                    size: AvatarImageSize::Medium,
                                                    User {}
                                                }
                                            }
                                        }
                                            {user.user_id.as_str()}
                                    }
                            }
                            }
                        }
                    }
                } else {
                    rsx!{}
                }
            }
            Input {
                onchange: move |e: FormEvent| search_term.set(e.value()),
                value: search_term,
                placeholder: "Search user",
            }
            div {
                class: Styles::search_results_list,
                for user in search_results.read().iter() {
                    {
                    let user_clone = user.clone();
                    let avatar_url = user.avatar_url.clone().unwrap_or_default();
                        rsx! {
                            Item {
                                class: Styles::search_results_card,
                                onclick: move | _ | {
                                    let mut selected = selected_users.write();
                                    if let Some(index) = selected.iter().position(|u| u.user_id == user_clone.user_id) {
                                        selected.remove(index);
                                    } else {
                                        selected.push(user_clone.clone());
                                    }
                                },
                                variant: ItemVariant::Outline,
                                ImageAvatar {
                                    src: "{avatar_url}",
                                    alt: "User profile picture",
                                    size: AvatarImageSize::Medium,
                                    User {}
                                }
                                p {
                                    {user.user_id.as_str()}
                                }
                            }
                        }
                    }
                }
            }
            Button {
                onclick: move | _ | {
                    spawn(async move {
                    let client = app_state.matrix.cloned().client().await.unwrap();
                    let mut request = CreateRoomRequest::new();
                    for user in selected_users.read().iter() {
                        let user_id = UserId::parse(user.user_id.clone()).unwrap();
                        request.invite.push(user_id);
                    }
                    let result = client.create_room(request).await;
                    if result.is_err() {
                        todo!("Proper error handling");
                        return;
                    }
                    let result = result.unwrap();
                      navigator().push(Route::Room { id: result.room_id().to_owned() });
                    });
                },
                {"Create chat".to_string()}
            }
        }
    }
}

#[component]
pub fn NewRoom() -> Element {
    let mut open_create_dm = use_signal(|| false);

    rsx! {
        div {
            DropdownMenu {
                DropdownMenuTrigger {
                        Plus {}
                }
                DropdownMenuContent {
                    DropdownMenuItem {
                        index: 0_usize,
                        value: "dm".to_string(),
                        on_select: move |_: String| {
                            open_create_dm.set(true);
                        },
                        { "Create chat".to_string() }
                    }
                    DropdownMenuItem {
                        index: 1_usize,
                        value: "space".to_string(),
                        on_select: |_: String| {
                            println!("Create space clicked");
                        },
                        { "Create room".to_string() }
                    }
                }
            }
            NewRoomModal { open: open_create_dm }
        }
    }
}
