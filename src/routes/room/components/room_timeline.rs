use std::sync::Arc;

use chrono::{DateTime, Local, TimeZone, Utc};
use dioxus::prelude::*;
use futures_util::StreamExt;
use matrix_sdk::ruma::OwnedRoomId;
use matrix_sdk_ui::{
    eyeball_im::Vector,
    timeline::{RoomExt, TimelineItem, VirtualTimelineItem},
};

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

    use_effect(move || {
        let matrix = state.matrix.cloned();
        let room_id = room_id.clone();

        spawn(async move {
            let client = matrix.client().await;

            let Some(client) = client else {
                eprintln!("Client not found");
                return;
            };

            current_user_id.set(Some(client.user_id().unwrap().to_string()));

            let Some(room) = client.get_room(&room_id) else {
                eprintln!("Room not found");
                return;
            };

            let timeline = Arc::new(room.timeline_builder().build().await.unwrap());

            let (initial_items, mut stream) = timeline.subscribe().await;
            messages.set(initial_items);

            let timeline_clone = timeline.clone();
            spawn(async move {
                if let Err(e) = timeline_clone.paginate_backwards(20).await {
                    eprintln!("Error paginating backwards: {:?}", e);
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
        div {..attributes,

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
                        render_timeline_event(content, &sender, is_me, &time_of_event)
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
        }
    }
}
