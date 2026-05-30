use crate::routes::router::Route;
use dioxus::prelude::*;
use dioxus_icons::lucide::{Bell, House, Settings, User};

#[css_module("/src/layouts/sidebar/style.css")]
struct Styles;

#[component]
pub fn Sidebar() -> Element {
    let current_route: Route = use_route();

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
                    NavItem {
                        to: Route::Profile,
                        icon: rsx! {
                            Bell { size: 24 }
                        },
                        active: matches!(current_route, Route::Profile),
                    }
                }
                div { style: "flex: 1" }
                div { class: Styles::sidebar_bottom,
                    NavItem {
                        to: Route::Profile,
                        icon: rsx! {
                            Settings { size: 24 }
                        },
                        active: false,
                    }
                    NavItem {
                        to: Route::Profile,
                        icon: rsx! {
                            User { size: 24 }
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
                    to: Route::Profile,
                    icon: rsx! {
                        Bell { size: 24 }
                    },
                    active: matches!(current_route, Route::Profile),
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
