use crate::components::button::{Button, ButtonVariant};
use crate::components::input::Input;
use crate::state;
use crate::state::app_state::AppState;
use crate::state::notifications::Notification;
use dioxus::prelude::*;
use matrix_sdk::encryption::recovery::RecoveryState;
use matrix_sdk::stream::StreamExt;

#[css_module("/src/layouts/encryption/style.css")]
struct Styles;

#[component]
fn RecoveryModal(on_close: EventHandler<()>) -> Element {
    let mut secret_key = use_signal(String::new);
    let app_state = use_context::<AppState>();

    let matrix = app_state.matrix.read().clone();
    
    let client_res = use_resource(move || {
        let matrix = matrix.clone();
        async move {
            let manager = matrix.read().await;
            manager.client()
        }
    });

    let Some(client_opt) = client_res.cloned() else {
        return rsx! {
            div { "Loading..." }
        };
    };

    let Some(client) = client_opt else {
        return rsx! { "No client available" };
    };

    rsx! {
        div {
            class: Styles::recovery_overlay,
            onclick: move |_| on_close.call(()),
            div {
                class: Styles::recovery_card,
                onclick: move |e| e.stop_propagation(),

                h2 { class: Styles::recovery_title, "Restore Encryption Keys" }
                p { class: Styles::recovery_description,
                    "Enter your recovery key to access your encrypted message history on this device."
                }

                Input {
                    oninput: move |e: FormEvent| secret_key.set(e.value()),
                    r#type: "password",
                    placeholder: "Recovery Key (e.g. Esdf-1234-...)",
                    value: "{secret_key}",
                }

                div { class: Styles::recovery_actions,
                    Button {
                        variant: ButtonVariant::Ghost,
                        onclick: move |_| on_close.call(()),
                        "Cancel"
                    }
                    Button {
                        onclick: move |_| {
                            let key = secret_key().clone();

                            let client_clone = client.clone();

                            spawn(async move {
                                let recovery = client_clone.encryption().recovery();
                                match recovery.recover(&key).await {
                                    Ok(_) => {
                                        debug!("Successfully recovered encryption keys");
                                        on_close.call(());
                                    }
                                    Err(e) => {
                                        error!("Failed to recover encryption keys: {}", e);
                                    }
                                }
                            });
                        },
                        "Restore"
                    }
                }
            }
        }
    }
}

#[component]
pub fn Encryption() -> Element {
    let app_state = use_context::<AppState>();
    let mut notifications = use_context::<state::notifications::NotificationsState>();
    let mut last_notified_state = use_signal(|| None::<RecoveryState>);
    let mut show_modal = use_signal(|| false);

    use_effect(move || {
        let matrix = app_state.matrix.read().clone();

        spawn(async move {
            let client = {
                let manager = matrix.read().await;
                manager.client()
            };

            if let Some(client) = client {
                let initial_state = client.encryption().recovery().state();

                if initial_state == RecoveryState::Incomplete
                    && last_notified_state() != Some(RecoveryState::Incomplete)
                {
                    let new_notification = Notification::new(
                        "Encryption keys missing",
                        "This device is missing the encryption keys. Please restore from backup to access encrypted messages.",
                        state::notifications::NotificationType::Warning
                    ).with_action(state::notifications::NotificationAction::Button {
                        label: "Restore".to_string(),
                        on_click: Callback::new(move |_| {
                            show_modal.set(true);
                        }),
                    });
                    notifications.push(new_notification);
                    last_notified_state.set(Some(RecoveryState::Incomplete));
                }

                let mut stream = client.encryption().recovery().state_stream();
                while let Some(update) = stream.next().await {
                    if update == RecoveryState::Incomplete
                        && last_notified_state() != Some(RecoveryState::Incomplete)
                    {
                        let new_notification = Notification::new(
                            "Encryption keys missing",
                            "This device is missing the encryption keys. Please restore from backup to access encrypted messages.",
                            state::notifications::NotificationType::Warning
                        ).with_action(state::notifications::NotificationAction::Button {
                            label: "Restore".to_string(),
                            on_click: Callback::new(move |_| {
                                show_modal.set(true);
                            }),
                        });
                        notifications.push(new_notification);
                        last_notified_state.set(Some(RecoveryState::Incomplete));
                    } else if update != RecoveryState::Incomplete {
                        last_notified_state.set(Some(update));
                    }
                }
            }
        });
    });

    rsx! {
        if show_modal() {
            RecoveryModal { on_close: move |_| show_modal.set(false) }
        }
        Outlet::<crate::routes::router::Route> {}
    }
}
