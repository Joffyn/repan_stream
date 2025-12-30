use std::sync::{Arc, Weak};

use anyhow::{Context, anyhow, bail};
use async_tungstenite::tungstenite;
use futures_util::{Stream, StreamExt, stream::futures_unordered};
use gst::prelude::*;
use gstreamer::{
    self as gst, Pipeline, StructureRef,
    bus::BusStream,
    glib::{self, GString},
};
use gstreamer_webrtc::{
    WebRTCSDPType, WebRTCSessionDescription, ffi::GstWebRTCDataChannel, gst::Message, gst_sdp,
};
use serde::{Deserialize, Serialize};
use tokio::sync::{
    Mutex, futures,
    mpsc::{self, UnboundedReceiver},
};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::pipeline_handler::RepanSink;

const STUN_SERVER: &str = "stun://stun.l.google.com:19302";

// upgrade weak reference or return
#[macro_export]
macro_rules! upgrade_weak {
    ($x:ident, $r:expr) => {{
        match $x.upgrade() {
            Some(o) => o,
            None => return $r,
        }
    }};
    ($x:ident) => {
        upgrade_weak!($x, ())
    };
}

// JSON messages we communicate with
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase", untagged)]
enum JsonMsg {
    Ice {
        candidate: String,
        #[serde(rename = "sdpMLineIndex")]
        sdp_mline_index: u32,
    },
    Sdp {
        #[serde(rename = "type")]
        type_: String,
        sdp: String,
    },
}

#[derive(Debug, Clone)]
pub struct UserConn(Arc<UserConnectionInner>);

#[derive(Debug, Clone)]
struct WeakConn(Weak<UserConnectionInner>);

#[derive(Debug)]
pub struct UserConnectionInner {
    pipeline: gst::Pipeline,
    webrtcbin: gst::Element,
    send_msg_tx: mpsc::UnboundedSender<tungstenite::Message>,
    pipeline_stream: Arc<Mutex<BusStream>>,
}
impl Drop for UserConnectionInner {
    fn drop(&mut self) {
        let _ = self.pipeline.set_state(gst::State::Null);
    }
}
impl std::ops::Deref for UserConn {
    type Target = UserConnectionInner;

    fn deref(&self) -> &UserConnectionInner {
        &self.0
    }
}
impl WeakConn {
    fn upgrade(&self) -> Option<UserConn> {
        self.0.upgrade().map(UserConn)
    }
}
impl UserConn {
    fn downgrade(&self) -> WeakConn {
        WeakConn(Arc::downgrade(&self.0))
    }

    pub fn new() -> Result<Self, anyhow::Error> {
        let pipeline = gst::parse::launch(
            "audiomixer name=mix ! audioconvert ! audioresample ! audio/x-raw,rate=48000,channels=2 ! opusenc ! rtpopuspay pt=111 ! webrtcbin. webrtcbin name=webrtcbin",
        )
        .unwrap()
        .downcast::<gst::Pipeline>()
        .unwrap();

        let webrtcbin = pipeline.by_name("webrtcbin").unwrap();

        webrtcbin.set_property_from_str("stun-server", STUN_SERVER);
        webrtcbin.set_property_from_str("bundle-policy", "max-bundle");

        let bus = pipeline.bus().unwrap();
        let mut pipeline_stream = bus.stream();

        let (send_msg_tx, send_msg_rx) = mpsc::unbounded_channel::<tungstenite::Message>();

        pipeline.call_async(|pipeline: &Pipeline| {
            pipeline.set_state(gst::State::Playing).unwrap();

            println!("Pipeline started!");
        });

        let conn = UserConn(Arc::new(UserConnectionInner {
            pipeline,
            webrtcbin,
            send_msg_tx,
            pipeline_stream: Arc::new(Mutex::new(pipeline_stream)),
        }));
        println!("User connection created");
        let (tx, rx) = mpsc::channel::<String>(128);

        let conn_clone = conn.downgrade();
        conn.webrtcbin
            .connect("on-data-channel", false, move |values| {
                let conn = upgrade_weak!(conn_clone, None);
                for v in values {
                    let t = v.value_type();
                    println!("Type of value in datachannel msg: {:?}", t);
                }
                let obj = values[1].get::<glib::Object>().unwrap();
                let dc = obj
                    .downcast::<gstreamer_webrtc::WebRTCDataChannel>()
                    .unwrap();

                //let sender = tx.clone();
                dc.connect_on_message_string(move |_, msg| {
                    if let Some(msg) = msg {
                        conn.parse_data_channel_msg(msg.to_string());
                    } else {
                        println!("Message incoming from datachannel was None");
                    }
                });
                let num = values.len();
                println!("Number of messages on-data-channel: {:?}", num);
                None
            });
        conn.webrtcbin
            .connect("on-new-transceiver", false, |values| {
                for v in values {
                    let t = v.type_();
                    println!("Type of value in new transeiver msg: {:?}", t);
                }
                let num = values.len();
                println!("Number of messages on-new-transceiver: {:?}", num);

                //let obj = values[1].get::<glib::Object>().unwrap();
                //let tr = obj
                //    .downcast::<gstreamer_webrtc::WebRTCRTPTransceiver>()
                //    .unwrap();
                //println!("Got transceiver");
                //let sender = tr.sender().unwrap();
                //println!("Unwrapped sender");
                None
            });

        conn.webrtcbin
            .connect("prepare-data-channel", false, |values| {
                for v in values {
                    let t = v.type_();
                    println!("Type of value in prepare-data-channel msg: {:?}", t);
                }
                let num = values.len();
                println!("Number of messages prepare-data-channel: {:?}", num);

                //let obj = values[1].get::<glib::Object>().unwrap();
                //let tr = obj
                //    .downcast::<gstreamer_webrtc::WebRTCRTPTransceiver>()
                //    .unwrap();
                //println!("Got transceiver");
                //let sender = tr.sender().unwrap();
                //println!("Unwrapped sender");
                None
            });

        conn.webrtcbin.connect("on-ice-candidate", false, |values| {
            for v in values {
                let t = v.type_();
                println!("Type of value in on-ice-candidate: {:?}", t);
            }
            let num = values.len();
            println!("Number of messages on-ice-candidate: {:?}", num);

            //let obj = values[1].get::<glib::Object>().unwrap();
            //let tr = obj
            //    .downcast::<gstreamer_webrtc::WebRTCRTPTransceiver>()
            //    .unwrap();
            //println!("Got transceiver");
            //let sender = tr.sender().unwrap();
            //println!("Unwrapped sender");
            None
        });
        //conn.webrtcbin
        //    .connect("on-negotiation-needed", false, |values| {
        //        for v in values {
        //            let t = v.type_();
        //            println!("Type of value in on-negotiation-needed: {:?}", t);
        //        }
        //        let num = values.len();
        //        println!("Number of messages on-negotiation-needed: {:?}", num);

        //        //let obj = values[1].get::<glib::Object>().unwrap();
        //        //let tr = obj
        //        //    .downcast::<gstreamer_webrtc::WebRTCRTPTransceiver>()
        //        //    .unwrap();
        //        //println!("Got transceiver");
        //        //let sender = tr.sender().unwrap();
        //        //println!("Unwrapped sender");
        //        None
        //    });

        Ok(conn)
    }

    pub async fn set_remote_description(&mut self, sdp_offer: &str) -> String {
        let pipeline = self.pipeline_stream.clone();
        let mut guard = pipeline.lock().await;
        let ret = gst_sdp::SDPMessage::parse_buffer(sdp_offer.as_bytes())
            .map_err(|_| anyhow!("Failed to parse SDP offer"))
            .unwrap();

        let offer = WebRTCSessionDescription::new(WebRTCSDPType::Offer, ret);

        loop {
            if let Some(msg) = guard.next().await {
                match msg.view() {
                    gst::MessageView::StateChanged(s) => {
                        if s.current() == gst::State::Playing {
                            self.webrtcbin.emit_by_name::<()>(
                                "set-remote-description",
                                &[&offer, &None::<gst::Promise>],
                            );

                            return "Played!".to_string();
                        }
                    }
                    _ => (),
                }
            }
        }
        drop(guard);
        "Did not pass".to_string()
    }
    pub async fn create_sdp_answer(&self) -> Option<String> {
        let conn_clone = self.downgrade();
        let (tx, rx) = tokio::sync::oneshot::channel::<String>();

        self.pipeline.call_async(move |_| {
            let conn = upgrade_weak!(conn_clone);
            let promise = gst::Promise::with_change_func(move |reply| {
                let answer = reply.unwrap().unwrap();
                if let Ok(e) = answer.get::<glib::Error>("error") {
                    eprintln!("Answer: {e}");
                } else {
                    let thing = answer
                        .value("answer")
                        .unwrap()
                        .get::<WebRTCSessionDescription>()
                        .unwrap();
                    //println!("{:?}", thing.sdp().to_string());

                    tx.send(thing.sdp().to_string());
                }
            });
            println!("Async was called");

            conn.webrtcbin
                .emit_by_name::<()>("create-answer", &[&None::<gst::Structure>, &promise]);
        });

        match rx.await {
            Ok(msg) => return Some(msg.to_string()),
            Err(e) => return None,
        }
        None
    }

    pub async fn set_local_description(&self, local_desc: String) -> Option<String> {
        let conn_clone = self.downgrade();
        let (tx, rx) = tokio::sync::oneshot::channel::<String>();

        self.pipeline.call_async(move |_| {
            let conn = upgrade_weak!(conn_clone);
            let promise = gst::Promise::with_change_func(move |reply| {
                tx.send("Set local description successfully".to_string());
            });
            let ret = gst_sdp::SDPMessage::parse_buffer(local_desc.as_bytes())
                .map_err(|_| anyhow!("Failed to parse local description"))
                .unwrap();

            let answer = WebRTCSessionDescription::new(WebRTCSDPType::Answer, ret);

            conn.webrtcbin
                .emit_by_name::<()>("set-local-description", &[&answer, &promise]);
        });

        match rx.await {
            Ok(msg) => return Some(msg.to_string()),
            Err(e) => return None,
        }
        None
    }
    pub async fn add_ice_candidate(&self, mlineindex: u32, candidate: String) {
        println!("Added ice candidate");
        self.webrtcbin
            .emit_by_name::<()>("add-ice-candidate", &[&mlineindex, &candidate]);
    }
    fn change_audio_src(&self, tracks: &[String]) -> Result<(), anyhow::Error> {
        for (index, track) in tracks.iter().enumerate() {
            let desc = format!(
                r#"filesrc location={track} !
                                wavparse !
                                audioconvert !
                                audioresample ! 
                                audio/x-raw,rate=48000,channels=2 !
                                audiopanorama name=pan{index} !
                                volume name=vol{index} !
                                queue name=q"#
            );

            let mixer = self.pipeline.by_name("mix").unwrap();
            let mix_pad = mixer.request_pad_simple("sink_%u").unwrap();
            let bin = gst::parse::bin_from_description(desc.as_str(), true).unwrap();
            self.pipeline.add(&bin);
            let src = bin.static_pad("src").unwrap();
            src.link(&mix_pad).unwrap();
            bin.sync_state_with_parent().unwrap();
        }
        Ok(())
    }
    fn parse_data_channel_msg(&self, unparsed_msg: String) {
        self.change_audio_src(&[
            "/home/joffy/Work/repan_stream/AnalogueIceSafezone.wav".to_string(),
            "/home/joffy/Work/repan_stream/3.wav".to_string(),
        ]);
    }
}
