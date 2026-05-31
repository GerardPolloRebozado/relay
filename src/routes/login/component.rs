use crate::components::button::Button;
use crate::components::card::*;
use crate::components::input::Input;
use crate::components::label::Label;
use crate::routes::router::Route;
use crate::state::app_state::AppState;
use dioxus::prelude::*;
use matrix_sdk::ruma::UserId;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct LoginValues {
    username: String,
    password: String,
}

#[css_module("/src/routes/login/style.css")]
struct Styles;

#[component]
pub fn Login() -> Element {
    let app_state = use_context::<AppState>();
    let mut logging_in = use_signal(|| false);

    use_effect(move || {
        if *logging_in.read() && *app_state.first_sync_done.read() {
            navigator().push(Route::Home);
        }
    });

    rsx! {
        div { class: Styles::login_page,
            Card { class: "login-card",
                CardHeader { class: "login-header",
                    CardTitle { class: "login-title", "Login" }
                }
                CardContent {
                    if *logging_in.read() {
                        div { class: "flex flex-col items-center justify-center p-8",
                            div { class: "loader mb-4" }
                            p { "Synchronizing your messages..." }
                        }
                    } else {
                        form {
                            class: Styles::login_form,
                            onsubmit: move |event| {
                                event.prevent_default();
                                let values = event.parsed_values();
                                if values.is_err() {
                                    println!("Client error");
                                    return;
                                }
                                let values: LoginValues = values.unwrap();
                                let username = UserId::parse(values.username);
                                if username.is_err() {
                                    println!("username error");
                                    return;
                                }
                                let username = username.unwrap();
spawn(async move {
    logging_in.set(true);
    let manager = app_state.matrix.cloned();

    match manager.login(&username, &values.password).await {
        Ok(_) => {
            let _ = manager.start_sync().await;
        }
        Err(e) => {
            println!("Login error: {}", e);
            logging_in.set(false);
        }
    }
});
                            },
                            div { class: Styles::form_group,
                                Label { html_for: "username", "Username" }
                                Input {
                                    r#type: "text",
                                    placeholder: "Enter your username",
                                    name: "username",
                                }
                            }
                            div { class: Styles::form_group,
                                Label { html_for: "password", "Password" }
                                Input {
                                    r#type: "password",
                                    placeholder: "Enter your password",
                                    name: "password",
                                }
                            }
                            Button { r#type: "submit", class: "login-btn", "Login" }
                        }
                    }
                }
            }
        }
    }
}
