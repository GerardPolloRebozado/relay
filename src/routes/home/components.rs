use crate::{
    components::{
        avatar::{AvatarImageSize, ImageAvatar},
        badge::{Badge, BadgeVariant},
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
use dioxus_router::components::Link;
use matrix_sdk::ruma::OwnedRoomId;

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
    let mut search_results = use_signal(|| Vec::new());
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
            search_results.write().clear();
            for user in response.results {
                if let Some(avatar_url) = user.avatar_url.clone() {
                    search_results.write().push(UserSearchResult {
                        user_id: user.user_id.clone().to_string(),
                        avatar_url: get_img(avatar_url).await,
                    });
                } else {
                    search_results.write().push(UserSearchResult {
                        user_id: user.user_id.clone().to_string(),
                        avatar_url: None,
                    });
                }
            }
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
                                    if let Some(avatar_url) = user.avatar_url.clone() {
                                        ImageAvatar {
                                            src: "{avatar_url}",
                                            alt: "User profile picture",
                                            size: AvatarImageSize::Medium,
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
                        rsx! {
                            Item {
                                class: Styles::search_results_card,
                                onclick: move | _ | selected_users.write().push(user_clone.clone()),
                                variant: ItemVariant::Outline,
                            if let Some(avatar_url) = user.avatar_url.clone() {
                                ImageAvatar {
                                    src: "{avatar_url}",
                                    alt: "User profile picture",
                                    size: AvatarImageSize::Medium,
                                }
                            }
                                p {
                                    {user.user_id.as_str()}
                                }
                            }
                        }
                    }
                }
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
