#![allow(warnings)]
#![warn(unused_variables)]

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
use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};
#[cfg(feature = "ssr")]
use tokio::sync::RwLock;
use uuid::Uuid;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    js_sys::{self, JsString, Reflect},
    MediaStream, RtcRtpTransceiver,
};
use web_sys::{RtcDataChannel, RtcRtpTransceiverInit};
use web_sys::{RtcDataChannelEvent, RtcPeerConnectionIceEvent};
use web_sys::{RtcPeerConnection, RtcTrackEvent};

use crate::backend::client_connections::{
    get_gst_sdp_answer, post_client_sdp_offer, post_ice_candidate_to_server, ClientMessage,
};

const STUN_SERVER: &str = "stun://stun.l.google.com:19302";
#[component]
pub fn OfferComp(user_id: ReadSignal<String>) -> impl IntoView {
    //let (p, s_p) = signal_local::<Option<RtcPeerConnection>>(None);
    let mut pc: Option<Rc<RtcPeerConnection>> = None;
    let mut dc: Option<Rc<RtcDataChannel>> = None;
    let mut tr: Option<Rc<RtcRtpTransceiver>> = None;
    let audio: NodeRef<leptos::html::Audio> = NodeRef::new();
    if cfg!(feature = "hydrate") {
        pc = Some(Rc::new(web_sys::RtcPeerConnection::new().unwrap()));
        dc = Some(Rc::new(pc.clone().unwrap().create_data_channel("channel")));
        let mut tr_init = RtcRtpTransceiverInit::new();
        tr_init.set_direction(web_sys::RtcRtpTransceiverDirection::Recvonly);

        tr = Some(Rc::new(
            pc.clone()
                .unwrap()
                .add_transceiver_with_str_and_init("audio", &tr_init),
        ));

        let audio_elem = document().create_element("audio").unwrap();
    }

    view! {
        <audio node_ref=audio controls autoplay></audio>
        <button on:click=move |_| {
            if cfg!(feature = "hydrate") {
                let pc_clone = pc.clone();
                let dc_clone = dc.clone();
                let tr_clone = tr.clone();
                let audio_ref = audio.clone();
                let pc_clone = pc_clone.unwrap().clone();
                let dc_clone = dc_clone.unwrap().clone();
                let tr_clone = tr_clone.unwrap().clone();

                spawn_local(async move {
                    let id = user_id.get();
                    use wasm_bindgen::{prelude::Closure, JsCast};

                    let track_callback = Closure::<dyn FnMut(_)>::new(move |ev: RtcTrackEvent| {
                        let audio_el = audio_ref.get().unwrap();
                        let streams = ev.streams();
                        let stream: MediaStream = streams.get(0).unchecked_into::<MediaStream>();
                        let _ = js_sys::Reflect::set(audio_el.as_ref(), &"srcObject".into(), stream.as_ref());
                        let _ = audio_el.play();

                        log!("On track callback called!");
                    });
                    pc_clone.set_ontrack(Some(track_callback.as_ref().unchecked_ref()));
                    track_callback.forget();

                    use crate::backend::client_connections::GstJsonMsg;
                    let dc_callback = Closure::<dyn FnMut(_)>::new(move |ev: RtcDataChannelEvent| {
                        log!("DC CALLBACK");
                    });
                    pc_clone.set_ondatachannel(Some(dc_callback.as_ref().unchecked_ref()));
                    dc_callback.forget();
                    let conn_callback = Closure::<dyn FnMut()>::new(move || {
                        log!("Conn CALLBACK");
                    });
                    pc_clone.set_onconnectionstatechange(Some(conn_callback.as_ref().unchecked_ref()));
                    conn_callback.forget();
                    let id_clone = id.clone();
                    let ice_callback = Closure::<dyn FnMut(_)>::new(move |ev: RtcPeerConnectionIceEvent| {
                    if let Some(candidate) = ev.candidate() {

                        log!("{:?}", candidate.candidate());
                        let mline = candidate.sdp_m_line_index().unwrap();
                        log!("{:?}", mline);
                        let id_clone = id_clone.clone();
                        spawn_local(async move {
                            log!("Trying to post ice candidate");
                            let _ = post_ice_candidate_to_server(id_clone.clone(), candidate.candidate(), mline as u32).await.unwrap();
                            });
                    }
                    });
                    pc_clone.set_onicecandidate(Some(ice_callback.as_ref().unchecked_ref()));
                    ice_callback.forget();


                    let dc_clone2 = dc_clone.clone();
                    let func = Closure::<dyn FnMut()>::new(move || {
                        log!("Sent from client");
                        dc_clone2.send_with_str("Message from datachannel!").unwrap();
                    });
                    dc_clone.clone().set_onopen(Some(func.as_ref().unchecked_ref()));
                    func.forget();

                    let offer = JsFuture::from(pc_clone.create_offer()).await.unwrap();
                    let sdp_offer = js_sys::JSON::stringify(&offer)
                        .unwrap()
                        .as_string()
                        .unwrap();

                    let json: GstJsonMsg = serde_json::from_str(sdp_offer.as_str()).unwrap();

                    if let GstJsonMsg::Sdp { sdp, r#type } = json {
                        log!("Parsed sdp in client to local desc");
                        let local = web_sys::RtcSessionDescriptionInit::new(web_sys::RtcSdpType::Offer);
                        local.set_sdp(sdp.as_str());
                        let local_promise = pc_clone.set_local_description(&local);
                        let _ = JsFuture::from(local_promise).await.unwrap();
                        log!("Attempting to post client sdp");

                        let _ = post_client_sdp_offer(id.clone(), sdp_offer)
                            .await
                            .unwrap();
                        log!("Posted client sdp");
                        let answer = get_gst_sdp_answer(id).await.unwrap();
                        log!("Got sdp from gstreamer");
                        let remote = web_sys::RtcSessionDescriptionInit::new(web_sys::RtcSdpType::Answer);
                        remote.set_sdp(answer.as_str());
                        log!("Created remote description");
                        let remote_promise = pc_clone.set_remote_description(&remote);
                        let _ = JsFuture::from(remote_promise).await.unwrap();
                    }
                });
            }
            }>
           "CONNECT"
        </button>
    }
}
