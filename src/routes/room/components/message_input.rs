use crate::{
    components::{button::Button, input::Input, label::Label},
    state::app_state::AppState,
};
use dioxus::{html::FileData, prelude::*};
use dioxus_icons::lucide::{Plus, Send};
use matrix_sdk::ruma::{
    OwnedRoomId,
    events::{AnyMessageLikeEventContent, room::message::RoomMessageEventContent},
};
use matrix_sdk_ui::timeline::{AttachmentConfig, AttachmentSource, RoomExt};

#[css_module("src/routes/room/components/message_input.css")]
struct Styles;

#[component]
pub fn MessageInput(
    #[props(extends=GlobalAttributes)] attributes: Vec<Attribute>,
    room_id: OwnedRoomId,
) -> Element {
    let state = use_context::<AppState>();

    let mut text_input = use_signal(String::new);
    let mut file_input_key = use_signal(|| 0);
    let mut selected_files = use_signal(Vec::<FileData>::new);

    rsx! {
        form {
            class: Styles::input_area,
            onsubmit: move |e: FormEvent| {
                e.prevent_default();
                let text_to_send = text_input();
                let files_to_send = selected_files();

                if text_to_send.trim().is_empty() && files_to_send.is_empty() {
                    return;
                }

                text_input.set(String::new());
                selected_files.set(Vec::new());
                file_input_key.with_mut(|k| *k += 1);

                let matrix = state.matrix.cloned();
                let room_id = room_id.clone();

                spawn(async move {
                    let client = matrix.client().await.unwrap();
                    let room = client.get_room(&room_id).unwrap();
                    let timeline = room.timeline().await.unwrap();
                    if !text_to_send.trim().is_empty() {
                        let _ = timeline
                            .send(
                                AnyMessageLikeEventContent::RoomMessage(
                                    RoomMessageEventContent::text_plain(text_to_send),
                                ),
                            )
                            .await;
                    }
                    for file in files_to_send {
                        let file_name = file.name();
                        if let Ok(bytes) = file.read_bytes().await {
                            let mime_type = mime_guess::from_path(
                                    std::path::Path::new(&file_name),
                                )
                                .first_or_octet_stream();
                            let source = AttachmentSource::Data {
                                filename: file_name,
                                bytes: bytes.to_vec(),
                            };
                            let config = AttachmentConfig::default();
                            let _ = timeline.send_attachment(source, mime_type, config).await;
                        }
                    }
                });
            },
            div { class: Styles::file_input_wrapper,
                Input {
                    key: "{file_input_key}",
                    r#type: "file",
                    name: "file",
                    id: "file",
                    class: Styles::file_input,
                    onchange: move |e: FormEvent| {
                        for file in e.files() {
                            selected_files.write().push(file);
                        }
                    },
                }
                Label { html_for: "file",
                    div { class: Styles::plus_button, Plus {} }
                }
            }
            Input {
                r#type: "text",
                placeholder: "Type a message...",
                name: "text",
                value: "{text_input}",
                oninput: move |e: Event<FormData>| text_input.set(e.value()),
            }
            Button {
                r#type: "submit",
                disabled: text_input().is_empty() && selected_files().is_empty(),
                Send {}
            }
        }
    }
}
