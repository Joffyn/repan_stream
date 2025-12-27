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

use crate::backend::client_connections::{
    get_gst_sdp_answer, post_client_sdp_offer, ClientMessage,
};

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
pub fn OfferComp(user_id: ReadSignal<String>) -> impl IntoView {
    let offer = LocalResource::new(move || get_sdp());
    //let answer = OnceResource::new(get_gst_sdp_answer(user_id.get()));

    view! {
        <button on:click=move |_| {
            spawn_local(async move {
                log!("REEEE");
                let offer = offer.await;
                log!("{:?}", offer);
                let _ = post_client_sdp_offer(user_id.get(), offer).await.unwrap();
                let answer = get_gst_sdp_answer(user_id.get()).await;
                log!("{:?}", answer);
            });
            }>
           "CONNECT"
        </button>
    }
}

#[component]
pub fn WebRtcComp(user_id: ReadSignal<String>) -> impl IntoView {
    let answer = OnceResource::new(get_gst_sdp_answer(user_id.get()));
    let offer_res = LocalResource::new(move || get_sdp());
    let sdp_action = Action::new(move |offer: &String| {
        let id = user_id.get();
        let offer = offer.clone();

        async move {
            let _ = post_client_sdp_offer(id.clone(), offer).await;
            log!("First complete");
            let answer = get_gst_sdp_answer(id.clone()).await;
            log!("second complete: {:?}", answer.unwrap());
        }
    });
    view! {
        //<div>
        //{
        //    move || {

        //        //spawn_local(async move {
        //        //    let id = user_id.get().as_u128();
        //        //    let _ = post_client_sdp_offer(id, offer.await).await;
        //        //    log!("First complete");
        //        //    let answer = get_gst_sdp_answer(id).await;
        //        //    log!("second complete: {:?}", answer.unwrap());

        //        //})
        //    }
        //}
        //</div>

    <Suspense fallback=move || {
        view! {
            <p>"Loading WebRTC..."</p>
        }
    }>
    {
        let offer = offer_res.get().unwrap();
        move || Suspend::new(async move {

                log!("NMEM");
              //sdp_action.dispatch(offer);

            //let sdp = answer.await.unwrap();
            //log!("{}", sdp.as_str());
            //view!{
            //    <p>{sdp}</p>
            //}
        })
    }
    </Suspense>

    }
}
#[cfg(feature = "hydrate")]
async fn get_sdp() -> String {
    //use leptos::tachys::dom::document;

    let pc = web_sys::RtcPeerConnection::new().unwrap();
    let dc = pc.create_data_channel("channel");

    //pc.add_track();

    let offer = JsFuture::from(pc.create_offer()).await.unwrap();
    let sdp = js_sys::JSON::stringify(&offer)
        .unwrap()
        .as_string()
        .unwrap();

    log!("{:?}", sdp);
    sdp
}
#[cfg(not(feature = "hydrate"))]
async fn get_sdp() -> String {
    "Null".to_string()
}
