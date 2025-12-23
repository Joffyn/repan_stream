use futures::{channel::mpsc, stream::SplitSink, SinkExt, StreamExt};
use leptos::{
    logging::{log, warn},
    prelude::*,
    reactive::spawn_local,
    server_fn::{codec::JsonEncoding, BoxedStream, Websocket},
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::sync::Arc;
#[cfg(feature = "ssr")]
use tokio::sync::RwLock;
use uuid::Uuid;
#[cfg(feature = "hydrate")]
use wasm_bindgen::JsValue;
#[cfg(feature = "hydrate")]
use wasm_bindgen_futures::JsFuture;
#[cfg(feature = "hydrate")]
use web_sys::js_sys::{self, JsString, Reflect};

const STUN_SERVER: &str = "stun://stun.l.google.com:19302";
//pub static GSTREAMER_SENDER: Lazy<Arc<RwLock<Option<SplitSink<WebSocket, Message>>>>> = Lazy::new(||

#[cfg(feature = "ssr")]
pub static SERVER_TO_SOCKET_HANDLE_SENDER: Lazy<
    Arc<RwLock<Option<tokio::sync::mpsc::Sender<String>>>>,
> = Lazy::new(|| Arc::new(RwLock::new(None)));
#[cfg(feature = "ssr")]
pub static SOCKET_HANDLE_TO_SERVER_RECEIVER: Lazy<
    Arc<RwLock<Option<tokio::sync::mpsc::Receiver<String>>>>,
> = Lazy::new(|| Arc::new(RwLock::new(None)));

#[component]
pub fn WebRtcComp() -> impl IntoView {
    let (mut to_server_tx, from_client_rx) = mpsc::channel(1);
    let user_id = Uuid::new_v4();

    //Websocket
    if cfg!(feature = "hydrate") {
        spawn_local(async move {
            match echo_websocket(from_client_rx.into()).await {
                Ok(mut messages) => {
                    while let Some(Ok(msg)) = messages.next().await {
                        log!("{:?}", serde_json::to_string(&msg));
                    }
                }
                Err(e) => warn!("{e}"),
                //_ => ()
            }
        });
    }

    let sdp_data = LocalResource::new(move || get_sdp());

    //Send to server
    view! {
        <p>{move || sdp_data.get()}</p>
            <button on:click=move |_|
            {
                let sdp = sdp_data.get().unwrap();
                let sdp_json = serde_json::from_str::<GstJsonMsg>(sdp.as_str()).unwrap();
                let msg = ClientMessage::Payload { id: user_id.as_u128().clone(), gst_msg: sdp_json};
                log!("{:?}", serde_json::to_string(&msg).unwrap());
                let _ = to_server_tx.clone().try_send(Ok(serde_json::to_string(&msg).unwrap()));

            }>
            "Test"
            </button>
    }
}
#[server(protocol = Websocket<JsonEncoding, JsonEncoding>)]
async fn echo_websocket(
    from_client_rx: BoxedStream<String, ServerFnError>,
) -> Result<BoxedStream<ClientMessage, ServerFnError>, ServerFnError> {
    let mut input = from_client_rx;
    let (mut to_client_tx, from_server_rx) = mpsc::channel(1);

    let to_gstreamer = SERVER_TO_SOCKET_HANDLE_SENDER.clone();

    tokio::spawn(async move {
        let _ = to_client_tx
            .send(Ok(ClientMessage::Test {
                test: "Meme".to_string(),
            }))
            .await;
        //Incoming from client
        while let Some(Ok(msg)) = input.next().await {
            let gstreamer_sender = {
                let guard = to_gstreamer.read().await;
                guard.as_ref().cloned()
            };

            if let Some(s) = gstreamer_sender {
                s.send(msg).await;
            } else {
                log!("No backend streamer connected!");
            }
        }
    });
    let from_gst = SOCKET_HANDLE_TO_SERVER_RECEIVER.clone();
    tokio::spawn(async move {
        let mut guard = from_gst.write().await;
        let guard = guard.as_mut().unwrap();
        //let _ = guard
        //   .send(msg.to_text().unwrap().to_string())
        //   .await
        //   .unwrap();
        while let Some(msg) = guard.recv().await {
            log!("Received from gst: {:?}", msg);
        }
    });

    Ok(from_server_rx.into())
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase", untagged)]
pub enum ClientMessage {
    Payload { gst_msg: GstJsonMsg, id: u128 },
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
#[cfg(feature = "hydrate")]
async fn get_sdp() -> String {
    use leptos::tachys::dom::document;

    let pc = web_sys::RtcPeerConnection::new().unwrap();
    let dc = pc.create_data_channel("channel");

    //pc.add_track();

    let offer = JsFuture::from(pc.create_offer()).await.unwrap();
    let sdp = js_sys::JSON::stringify(&offer)
        .unwrap()
        .as_string()
        .unwrap();

    //let sdp = offer.as_string().unwrap();

    log!("{:?}", sdp);
    sdp
}
#[cfg(not(feature = "hydrate"))]
async fn get_sdp() -> String {
    "MEME".to_string()
}
