use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Local, TimeZone, Utc};
use dioxus::document::eval;
use dioxus::prelude::*;
use futures_util::StreamExt;
use matrix_sdk::ruma::{OwnedEventId, OwnedRoomId};
use matrix_sdk::{event_cache::PaginationStatus, ruma::OwnedUserId};
use matrix_sdk_ui::{
    eyeball_im::Vector,
    timeline::{RoomExt, TimelineItem, VirtualTimelineItem},
};

use crate::routes::room::components::ReadReceipt;
use crate::routes::room::settings::components::participants;
use crate::{routes::room::timeline_utilities::render_timeline_event, state::app_state::AppState};

#[css_module("src/routes/room/components/room_timeline.css")]
struct Styles;

#[component]
pub fn RoomTimeline(
    room_id: OwnedRoomId,
    #[props(extends=GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let state = use_context::<AppState>();
    let mut messages = use_signal(Vector::<Arc<TimelineItem>>::default);
    let mut current_user_id = use_signal(|| None::<String>);
    let mut pagination_status = use_signal(|| PaginationStatus::Idle {
        hit_timeline_start: false,
    });
    let mut last_messages_read = use_signal(HashMap::<OwnedEventId, Vec<OwnedUserId>>::new);

    use_effect(move || {
        let matrix = state.matrix.cloned();
        let room_id = room_id.clone();

        spawn(async move {
            let client = matrix.client().await;

            let Some(client) = client else {
                error!("Client not found");
                return;
            };

            current_user_id.set(Some(client.user_id().unwrap().to_string()));

            let Some(room) = client.get_room(&room_id) else {
                error!("Room not found");
                return;
            };

            let timeline = Arc::new(room.timeline_builder().build().await.unwrap());

            if let Ok(participants) = room.joined_user_ids().await {
                for p in participants {
                    let read_receipt = timeline.latest_user_read_receipt(&p).await;
                    if read_receipt.is_none() {
                        continue;
                    }
                    let receipt = read_receipt.unwrap();
                    let user_id_clone = p.clone();
                    let event_id_clone = &receipt.0.clone();
                    if last_messages_read.read().contains_key(&receipt.0.clone()) {
                        let mut existing_users = last_messages_read.read()[event_id_clone].clone();
                        existing_users.push(user_id_clone);

                        last_messages_read
                            .write()
                            .insert(event_id_clone.clone(), existing_users);
                    } else {
                        last_messages_read
                            .write()
                            .insert(receipt.0.clone(), vec![user_id_clone]);
                    }
                }
            }

            if let Some((initial_status, mut status_stream)) =
                timeline.live_back_pagination_status().await
            {
                pagination_status.set(initial_status);
                let mut pagination_status_clone = pagination_status;
                spawn(async move {
                    while let Some(status) = status_stream.next().await {
                        pagination_status_clone.set(status);
                    }
                });
            }

            let (initial_items, mut stream) = timeline.subscribe().await;
            messages.set(initial_items);

            let timeline_clone = timeline.clone();
            spawn(async move {
                if let Err(e) = timeline_clone.paginate_backwards(20).await {
                    error!("Error paginating backwards: {:?}", e);
                }
            });

            let mut js_eval = eval(
                r#"
                const el = document.getElementById("room-timeline");
                if (el) {
                    el.addEventListener("scroll", () => {
                        const threshold = 50;
                        const distanceToTop = el.scrollHeight - el.clientHeight + el.scrollTop;
                        if (el.scrollTop < -10 && distanceToTop < threshold) {
                            dioxus.send("trigger");
                        }
                    });
                }
            "#,
            );

            let timeline_for_scroll = timeline.clone();
            spawn(async move {
                while let Ok(_msg) = js_eval.recv::<String>().await {
                    let current_status = *pagination_status.read();
                    if let PaginationStatus::Idle {
                        hit_timeline_start: false,
                    } = current_status
                    {
                        let timeline_back = timeline_for_scroll.clone();
                        spawn(async move {
                            if let Err(e) = timeline_back.paginate_backwards(10).await {
                                error!("Error paginating backwards: {:?}", e);
                            }
                        });
                    }
                }
            });

            while let Some(diff_batch) = stream.next().await {
                let mut msgs = messages.write();
                for diff in diff_batch {
                    diff.apply(&mut *msgs);
                }
            }
        });
    });

    rsx! {
        div { id: "room-timeline", ..attributes,

            for item in messages.read().iter().rev() {
                {
                    if let Some(event) = item.as_event() {
                        let sender = event.sender().to_string();
                        let content = event.content();

                        let timestamp = event.timestamp();
                        let datetime: DateTime<Local> = Local
                            .timestamp_millis_opt(timestamp.0.into())
                            .single()
                            .unwrap();
                        let time_of_event = datetime.format("%H:%M").to_string();
                        let user_id = current_user_id.read().clone().unwrap_or("null".to_string());
                        let is_me = user_id == sender;

                        let event_id = event.event_id().map(|id| id.to_owned());
                        let user_ids = event_id
                            .and_then(|id| { last_messages_read.read().get(&id).cloned() });
                        rsx! {
                            if let Some(ids) = user_ids {
                                div { class: Styles::receipt_container,
                                    for id in ids {
                                        ReadReceipt { key: "{id}", user_id: id }
                                    }
                                }
                            }
                            {render_timeline_event(content, &sender, is_me, &time_of_event)}
                        }
                    } else if item.is_date_divider() {
                        let date_text = if let Some(VirtualTimelineItem::DateDivider(ts)) = item
                            .as_virtual()
                        {
                            Utc.timestamp_millis_opt(ts.0.into())
                                .single()
                                .map(|d| d.format("%B %e, %Y").to_string())
                                .unwrap_or_else(|| "Unknown Date".to_string())
                        } else {
                            "Date Divider".to_string()
                        };
                        rsx! {
                            div { class: Styles::date_divider, "{date_text}" }
                        }
                    } else {
                        rsx! {
                            div { class: Styles::other_events, "Other event" }
                        }
                    }
                }
            }

            if let PaginationStatus::Paginating = *pagination_status.read() {
                div { class: Styles::loading_indicator, "Loading older messages..." }
            }
        }
    }
}
