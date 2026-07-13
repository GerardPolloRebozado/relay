use crate::{
    components::avatar::{AvatarImageSize, AvatarShape, ImageAvatar},
    custom_types::rooms::SpaceInfo,
    routes::{home::dm_utilities::get_room_avatar, router::Route},
    state::app_state::AppState,
};
use dioxus::prelude::*;
use dioxus_icons::lucide::{House, Settings};
use futures_util::{StreamExt, pin_mut};
use matrix_sdk_ui::room_list_service::filters::new_filter_space;

#[css_module("/src/layouts/sidebar/style.css")]
struct Styles;

#[component]
pub fn Sidebar() -> Element {
    let current_route: Route = use_route();
    let mut state = use_context::<AppState>();
    let mut space_list = use_signal(move || state.space_list.read().clone());

    use_future(move || async move {
        let matrix = state.matrix.cloned();

        let (client, room_list_service) = (matrix.client().await, matrix.room_list_service().await);

        let (Some(client), Some(room_list_service)) = (client, room_list_service) else {
            navigator().push(Route::Login);
            return;
        };
        let all_rooms_list = match room_list_service.all_rooms().await {
            Ok(list) => list,
            Err(e) => {
                error!("Failed to get all_rooms: {:?}", e);
                return;
            }
        };
        let (room_list_stream, controller) = all_rooms_list.entries_with_dynamic_adapters(10);
        pin_mut!(room_list_stream);

        // show spaces
        controller.set_filter(Box::new(Box::new(new_filter_space())));

        async fn to_space_info(client: &matrix_sdk::Client, room: matrix_sdk::Room) -> SpaceInfo {
            let display_name = room.display_name().await;
            let name = match display_name {
                Ok(dn) => dn.to_string(),
                Err(_) => "Unknown Space".to_string(),
            };
            let avatar_url = get_room_avatar(client, &room)
                .await
                .unwrap_or(String::new());
            SpaceInfo {
                id: room.room_id().to_owned(),
                name,
                avatar_url,
            }
        }

        while let Some(diffs) = room_list_stream.next().await {
            for diff in diffs {
                match diff {
                    matrix_sdk_ui::eyeball_im::VectorDiff::Reset { values } => {
                        let mut new_list = Vec::new();
                        for item in values {
                            new_list.push(to_space_info(&client, item.into_inner()).await);
                        }
                        space_list.set(new_list);
                    }
                    matrix_sdk_ui::eyeball_im::VectorDiff::PushFront { value } => {
                        let info = to_space_info(&client, value.into_inner()).await;
                        space_list.write().insert(0, info);
                    }
                    matrix_sdk_ui::eyeball_im::VectorDiff::PushBack { value } => {
                        let info = to_space_info(&client, value.into_inner()).await;
                        space_list.write().push(info);
                    }
                    matrix_sdk_ui::eyeball_im::VectorDiff::Insert { index, value } => {
                        let info = to_space_info(&client, value.into_inner()).await;
                        let mut list = space_list.write();
                        if index <= list.len() {
                            list.insert(index, info);
                        } else {
                            list.push(info);
                        }
                    }
                    matrix_sdk_ui::eyeball_im::VectorDiff::Set { index, value } => {
                        let info = to_space_info(&client, value.into_inner()).await;
                        let mut list = space_list.write();
                        if index < list.len() {
                            list[index] = info;
                        }
                    }
                    matrix_sdk_ui::eyeball_im::VectorDiff::Remove { index } => {
                        let mut list = space_list.write();
                        if index < list.len() {
                            list.remove(index);
                        }
                    }
                    matrix_sdk_ui::eyeball_im::VectorDiff::PopFront => {
                        let mut list = space_list.write();
                        if !list.is_empty() {
                            list.remove(0);
                        }
                    }
                    matrix_sdk_ui::eyeball_im::VectorDiff::PopBack => {
                        space_list.write().pop();
                    }
                    matrix_sdk_ui::eyeball_im::VectorDiff::Clear => {
                        space_list.set(Vec::new());
                    }
                    matrix_sdk_ui::eyeball_im::VectorDiff::Truncate { length } => {
                        space_list.write().truncate(length);
                    }
                    matrix_sdk_ui::eyeball_im::VectorDiff::Append { values } => {
                        for item in values {
                            let info = to_space_info(&client, item.into_inner()).await;
                            space_list.write().push(info);
                        }
                    }
                }
            }
        }
    });

    use_effect(move || {
        let list = space_list.read().clone();
        state.space_list.set(list);
    });

    rsx!(
        div { class: Styles::app_layout,
            // Desktop Sidebar
            aside { class: Styles::desktop_sidebar,
                div { class: Styles::sidebar_top,
                    NavItem {
                        to: Route::Home,
                        icon: rsx! {
                            House { size: 24 }
                        },
                        active: matches!(current_route, Route::Home),
                    }
                }
                div { class: Styles::space_list,
                    for space in space_list.iter() {
                        SpaceIcon { space: space.clone() }
                    }
                }
                div { class: Styles::sidebar_bottom,
                    NavItem {
                        to: Route::Settings,
                        icon: rsx! {
                            Settings { size: 24 }
                        },
                        active: false,
                    }
                }
            }

            // Main Content Area
            div { class: Styles::main_content, Outlet::<Route> {} }

            // Mobile Bottom Bar
            nav { class: Styles::mobile_bottom_bar,
                NavItem {
                    to: Route::Home,
                    icon: rsx! {
                        House { size: 24 }
                    },
                    active: matches!(current_route, Route::Home),
                }
                NavItem {
                    to: Route::Settings,
                    icon: rsx! {
                        Settings { size: 24 }
                    },
                    active: matches!(current_route, Route::Settings),
                }
            }
        }
    )
}

#[component]
fn NavItem(to: Route, icon: Element, active: bool) -> Element {
    let class = if active {
        format!("{} {}", Styles::nav_item, Styles::active)
    } else {
        Styles::nav_item.to_string()
    };
    rsx! {
        Link { to, class: "{class}", {icon} }
    }
}

#[component]
fn SpaceIcon(space: SpaceInfo) -> Element {
    let route: Route = use_route();
    let mut role = "link";

    if let Route::SpacePage { id } = route
        && id == space.id
    {
        role = "current_page";
    }
    rsx! {
        div {
            onclick: move |_| {
                navigator()
                    .push(Route::SpacePage {
                        id: space.id.clone(),
                    });
            },
            ImageAvatar {
                role,
                class: Styles::space_icon,
                size: AvatarImageSize::Medium,
                shape: AvatarShape::Rounded,
                src: space.avatar_url,
                {space.name.get(0..1)}
            }
        }
    }
}
