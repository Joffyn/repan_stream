use futures::{SinkExt, StreamExt, channel::mpsc, stream::SplitSink};
use leptos::{logging::{log, warn}, prelude::*, reactive::spawn_local, server_fn::{BoxedStream, Websocket, codec::JsonEncoding}};
#[cfg(feature = "hydrate")]
use wasm_bindgen::JsValue;
#[cfg(feature = "hydrate")]
use wasm_bindgen_futures::JsFuture;
#[cfg(feature = "hydrate")]
use web_sys::js_sys::{self, JsString, Reflect};
use once_cell::sync::Lazy;
use std::sync::{Arc};
#[cfg(feature = "ssr")]
use tokio::sync::RwLock;

const STUN_SERVER: &str = "stun://stun.l.google.com:19302";
//pub static GSTREAMER_SENDER: Lazy<Arc<RwLock<Option<SplitSink<WebSocket, Message>>>>> = Lazy::new(|| 

#[cfg(feature = "ssr")]
pub static GSTREAMER_SENDER: Lazy<Arc<RwLock<Option<tokio::sync::mpsc::Sender<String>>>>> = Lazy::new(|| 
{
    Arc::new(RwLock::new(None))
});

#[component]
pub fn WebRtcComp() -> impl IntoView
{
    let (mut tx, rx) = mpsc::channel(1);

    //Start websocket
    if cfg!(feature = "hydrate")
    {
        spawn_local(async move 
            {
                match echo_websocket(rx.into()).await
                {
                    Ok(mut messages) =>
                    {
                        while let Some(Ok(msg)) = messages.next().await
                        {
                            log!("{:?}", msg);
                        }
                    }
                    Err(e) => warn!("{e}"),
                    //_ => ()
                }
            });
    }

    let sdp_data = LocalResource::new(move || get_sdp());

    view!
    {
        <p>{move || sdp_data.get()}</p>
            <button on:click=move |_| 
            {
                let _ = tx.clone().try_send(Ok(sdp_data.get().unwrap()));

            }>
            "Test"
            </button>
    }
}
#[server(protocol = Websocket<JsonEncoding, JsonEncoding>)]
async fn echo_websocket(input: BoxedStream<String, ServerFnError>)
-> Result<BoxedStream<String, ServerFnError>, ServerFnError>
{
    let mut input = input;
    let (mut tx, rx) = mpsc::channel(1);

    let socket_sender = GSTREAMER_SENDER.clone();

    tokio::spawn(async move 
        {
            let _ = tx.send(Ok("Client connected to server".to_owned())).await;
            while let Some(Ok(msg)) = input.next().await
            {
                let sender = 
                {
                    let guard = socket_sender.read().await;
                    guard.as_ref().cloned()
                };

                if let Some(s) = sender
                {
                    s.send(msg).await;
                }
                else 
                {
                    log!("No backend streamer connected!");   
                }
            }
        });

    Ok(rx.into())

}
#[cfg(feature = "hydrate")]
async fn get_sdp() -> String
{
    let pc = web_sys::RtcPeerConnection::new().unwrap();
    let dc = pc.create_data_channel("channel");
    let offer =  JsFuture::from(pc.create_offer()).await.unwrap();
    let sdp = Reflect::get(&offer, &JsValue::from_str("sdp"))
        .unwrap()
        .as_string()
        .unwrap();

    sdp

}
#[cfg(not(feature = "hydrate"))]
async fn get_sdp() -> String
{
    "MEME".to_string()
}
