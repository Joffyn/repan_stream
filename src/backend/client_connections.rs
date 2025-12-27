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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase", untagged)]
pub enum ClientMessage {
    Payload { gst_msg: GstJsonMsg, id: String },
    Test { test: String },
}

// JSON messages we communicate with
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase", untagged)]
enum GstJsonMsg {
    Ice {
        candidate: String,
        #[serde(rename = "sdpMLineIndex")]
        sdp_mline_index: u32,
    },
    Sdp {
        sdp: String,
        #[serde(rename = "type")]
        type_: String,
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
    let msg = ClientMessage::Payload {
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
    //let (mut client_tx, mut client_rx) = tokio::sync::mpsc::channel::<String>(128);
    //let (mut gst_server_client_sender, gst_server_client_sink) =
    //    tokio::sync::mpsc::channel::<String>(128);

    //let streamer_to_server = SOCKET_HANDLE_TO_SERVER_RECEIVER.clone();
    //*streamer_to_server.write().await = Some(gst_server_client_sink);

    let (mut gst_sink, mut gst_receiver) = socket.split();
    let client_messager = GST_MSG_SINK.clone();
    *client_messager.lock().await = Some(gst_sink);

    tokio::spawn(handle_streamer_messages(gst_receiver));

    //tokio::spawn(handle_client_messages(streamer_sender, client_rx));
    //tokio::spawn(handle_streamer_messages(
    //    gst_server_client_sender,
    //    streamer_receiver,
    //));
    //log!("Socket disconnecting");
}
//async fn handle_client_messages(mut tx: SplitSink<WebSocket, Message>, mut rx: Receiver<String>) {
//    use axum::extract::ws::Utf8Bytes;
//    loop {
//        match rx.recv().await {
//            Some(msg) => {
//                //let json = serde_json::to_string(&msg).unwrap();
//                //log!("Sent: {} from server!", msg.as_str());
//                let _ = tx.send(Message::Text(Utf8Bytes::from(msg.as_str()))).await;
//            }
//            _ => (),
//        }
//    }
//}
#[cfg(feature = "ssr")]
async fn handle_streamer_messages(mut rx: SplitStream<WebSocket>) {
    loop {
        match rx.next().await {
            Some(Ok(msg)) => {
                //let gstreamer_receiver = SERVER_TO_SOCKET_HANDLE_SENDER.clone();
                //let guard = gstreamer_receiver.read().await;
                //let guard = guard.as_ref().unwrap();
                //let _ = guard
                //    .send(msg.to_text().unwrap().to_string())
                //    .await
                //    .unwrap();

                log!("From Gstreamer: {:?}", msg.to_text().unwrap());
                let client_connections = CLIENT_SDP_ANSWERS.clone();
                let mut guard = client_connections.lock().await;
                let _ = guard.insert("1".to_string(), msg.to_text().unwrap().to_string());

                //let _ = tx.send(msg.to_text().unwrap().to_string()).await.unwrap();
                //log!("From Gstreamer: {:?}", msg.to_text().unwrap());

                //drop(guard);
            }
            //log!("{:?}", msg.to_text().unwrap()),
            Some(Err(e)) => log!("{:?}", e.to_string()),
            None => {
                log!("Socket ended");
                break;
            }
            _ => println!("Server message not picked up"),
        }
    }
}
