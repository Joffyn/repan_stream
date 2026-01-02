#[cfg(feature = "ssr")]
use std::{collections::HashMap, sync::Arc};

#[cfg(feature = "ssr")]
use axum::extract::ws::{Message, Utf8Bytes, WebSocket};
#[cfg(feature = "ssr")]
use axum::extract::WebSocketUpgrade;
#[cfg(feature = "ssr")]
use axum::response::IntoResponse;
#[cfg(feature = "ssr")]
use futures::stream::{SplitSink, SplitStream};
#[cfg(feature = "ssr")]
use futures::{SinkExt, StreamExt};
use leptos::logging::log;
use leptos::prelude::guards;
use leptos::{prelude::ServerFnError, server};

#[cfg(feature = "ssr")]
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use tokio::sync::mpsc::{Receiver, Sender};
#[cfg(feature = "ssr")]
use tokio::sync::Mutex;
#[cfg(feature = "ssr")]
use tokio::time::{sleep_until, Duration, Instant};

#[cfg(feature = "ssr")]
pub static CLIENT_SDP_ANSWERS: Lazy<Arc<Mutex<HashMap<String, String>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

#[cfg(feature = "ssr")]
pub static GST_MSG_SINK: Lazy<Arc<Mutex<Option<SplitSink<WebSocket, Message>>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

//#[derive(Serialize, Deserialize)]
//#[serde(rename_all = "lowercase", untagged)]
//pub enum ClientMessage {
//    Payload { gst_msg: GstJsonMsg, id: String },
//    Test { test: String },
//}
#[derive(Debug, Serialize, Deserialize)]
pub struct ClientMessage {
    pub gst_msg: GstJsonMsg,
    id: String,
}

// JSON messages we communicate with
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", untagged)]
pub enum GstJsonMsg {
    Ice {
        candidate: String,
        #[serde(rename = "sdpMLineIndex")]
        sdp_mline_index: u32,
    },
    Sdp {
        sdp: String,
        #[serde(rename = "type")]
        r#type: String,
    },
    ChangeJam {
        path: String,
        date: String,
        tracks: Vec<String>,
    },
}

#[server(GetGstSdpAnswer)]
pub async fn get_gst_sdp_answer(id: String) -> Result<String, ServerFnError> {
    let mut sdp: Option<String> = None;
    loop {
        let sdp_map = CLIENT_SDP_ANSWERS.clone();
        let guard = sdp_map.lock().await;
        if guard.contains_key(&id) {
            sdp = Some(guard.get(&id).unwrap().to_owned());
            drop(guard);
            break;
        }
        sleep_until(Instant::now() + Duration::from_millis(100)).await;
    }
    Ok(sdp.unwrap())
}
#[server(PostClientSdpOffer)]
pub async fn post_client_sdp_offer(id: String, offer: String) -> Result<(), ServerFnError> {
    log!("Sent from server");
    let sdp_json = serde_json::from_str::<GstJsonMsg>(offer.as_str()).unwrap();
    let msg = ClientMessage {
        id,
        gst_msg: sdp_json,
    };
    let msg = serde_json::to_string(&msg).unwrap();

    let msg = Message::Text(Utf8Bytes::from(msg));
    let gst_sink = GST_MSG_SINK.clone();
    let mut guard = gst_sink.lock().await;
    let guard = guard.as_mut().unwrap();

    let _ = guard.send(msg).await;
    drop(guard);

    Ok(())
}
#[server(PostICECandidateToServer)]
pub async fn post_ice_candidate_to_server(
    id: String,
    candidate: String,
    mline: u32,
) -> Result<(), ServerFnError> {
    log!("Attempting to send ice from server");
    let ice = GstJsonMsg::Ice {
        candidate,
        sdp_mline_index: mline,
    };
    let msg = ClientMessage { gst_msg: ice, id };
    let msg = serde_json::to_string(&msg).unwrap();

    let msg = Message::Text(Utf8Bytes::from(msg));
    let gst_sink = GST_MSG_SINK.clone();
    let mut guard = gst_sink.lock().await;
    let guard = guard.as_mut().unwrap();

    let _ = guard.send(msg).await;
    drop(guard);

    log!("Sent Ice candidate from client");
    Ok(())
}

#[cfg(feature = "ssr")]
pub async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    log!("Connected!");
    ws.on_failed_upgrade(|e| eprintln!("{:?}", e))
        .on_upgrade(handle_socket)
}
#[cfg(feature = "ssr")]
async fn handle_socket(mut socket: WebSocket) {
    use axum::extract::ws::Utf8Bytes;

    socket.send(Message::Text(Utf8Bytes::from_static("Hello!")));
    log!("Socket being handled");
    let (mut gst_sink, mut gst_receiver) = socket.split();
    let client_messager = GST_MSG_SINK.clone();
    *client_messager.lock().await = Some(gst_sink);

    tokio::spawn(handle_streamer_messages(gst_receiver));
}
#[cfg(feature = "ssr")]
async fn handle_streamer_messages(mut rx: SplitStream<WebSocket>) {
    loop {
        match rx.next().await {
            Some(Ok(msg)) => {
                log!("From Gstreamer: {:?}", msg.to_text().unwrap());
                let text = msg.to_text().unwrap();
                let json: Result<ClientMessage, serde_json::Error> = serde_json::from_str(text);

                match json {
                    Ok(msg) => match msg.gst_msg {
                        GstJsonMsg::Sdp { r#type, sdp } => {
                            if r#type.ne("answer") {
                                log!("Got sdp, but it was offer");
                                continue;
                            }

                            let client_connections = CLIENT_SDP_ANSWERS.clone();
                            let mut guard = client_connections.lock().await;
                            let _ = guard.insert(msg.id.clone(), sdp);
                        }
                        _ => (),
                    },
                    _ => log!("Not json message"),
                }
            }
            Some(Err(e)) => log!("{:?}", e.to_string()),
            None => {
                log!("Socket ended");
                break;
            }
            _ => println!("Server message not picked up"),
        }
    }
}
