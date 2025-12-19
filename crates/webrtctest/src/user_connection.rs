use std::sync::{Arc, Weak};

use anyhow::{Context, anyhow, bail};
use async_tungstenite::tungstenite;
use futures_util::{Stream, stream::futures_unordered};
use gstreamer::{self as gst, Pipeline, StructureRef, glib::{self, GString}};
use gst::prelude::*;
use gstreamer_webrtc::{WebRTCSDPType, WebRTCSessionDescription, gst::Message, gst_sdp};
use serde::{Deserialize, Serialize};
use tokio::{sync::{futures, mpsc::{self, UnboundedReceiver}}};
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
pub struct UserConnectionInner
{
    pipeline: gst::Pipeline,
    webrtcbin: gst::Element,
    send_msg_tx: mpsc::UnboundedSender<tungstenite::Message>,
}
impl Drop for UserConnectionInner
{
    fn drop(&mut self)
    {
        let _ = self.pipeline.set_state(gst::State::Null);
    }
}
impl std::ops::Deref for UserConn
{
    type Target = UserConnectionInner;

    fn deref(&self) -> &UserConnectionInner
    {
        &self.0
    }
}
impl WeakConn
{
    fn upgrade(&self) -> Option<UserConn>
    {
        self.0.upgrade().map(UserConn)
    }
    
}
impl UserConn
{
    fn downgrade(&self) -> WeakConn
    {
        WeakConn(Arc::downgrade(&self.0))
    }


    pub fn new() -> Result<Self, anyhow::Error>
    {

        let pipeline = gst::parse::launch(
            "audiotestsrc is-live=true ! opusenc ! rtpopuspay pt=97 ! webrtcbin. webrtcbin name=webrtcbin")
            .unwrap()
            .downcast::<gst::Pipeline>()
            .unwrap();

        let webrtcbin = pipeline.by_name("webrtcbin").unwrap();

        webrtcbin.set_property_from_str("stun-server", STUN_SERVER);
        webrtcbin.set_property_from_str("bundle-policy", "max-bundle");

        let bus = pipeline.bus().unwrap();
        let send_gst_msg_rx = bus.stream();

        
        let (send_ws_msg_tx, send_ws_msg_rx) = mpsc::unbounded_channel::<tungstenite::Message>();


        pipeline.call_async(|pipeline: &Pipeline|
        {
            pipeline
                .set_state(gst::State::Playing)
                .unwrap();
        
              println!("Pipeline started!");
        });



        //let promise = gst::Promise::with_change_func(move |reply| 
        //{
        //    //let conn = upgrade_weak!(conn_clone);
        //    let answer = reply
        //        .unwrap()
        //        .unwrap();
        //    
        //    if let Ok(e) = answer.get::<glib::Error>("error")
        //    {
        //        eprintln!("Answer: {e}");
        //    }
        //});
        //webrtcbin.emit_by_name::<()>("create-answer", &[&None::<gst::Structure>, &promise]);

        let conn = UserConn(Arc::new(UserConnectionInner 
        {
            pipeline,
            webrtcbin,
            send_msg_tx: send_ws_msg_tx
        }));
        println!("User connection created");
        Ok(conn)
    }
    pub fn deliver_sdp(&self, sdp_offer: &str) -> Result<(), anyhow::Error>
    {
        let ret = gst_sdp::SDPMessage::parse_buffer(sdp_offer.as_bytes())
            .map_err(|_| anyhow!("Failed to parse SDP offer"))?;
        let clone = self.downgrade();
        self.pipeline.call_async(move |_| 
            {
                let real = upgrade_weak!(clone);

                let offer = WebRTCSessionDescription::new(
                    WebRTCSDPType::Offer,
                    ret,
                );

                println!("Remote description set");
                real.0.webrtcbin.emit_by_name::<()>("set-remote-description", &[&offer, &None::<gst::Promise>]);
            });
        println!("SDP offer delivered");
        Ok(())

    }
    pub async fn get_sdp_answer(&self) -> Result<String, anyhow::Error>
    {
        //self.webrtcbin.emit_by_name::<()>("create-answer", &[&None::<gst::Structure>, &None::<gst::Promise>]);

        //let offer: WebRTCSessionDescription = reply
        //    .unwrap()
        //    .value("offer")
        //    .unwrap()
        //    .get::<WebRTCSessionDescription>()
        //    .unwrap();

        //println!("On offer created!");
        //self.webrtcbin
        //    .emit_by_name::<()>("set-local-description", &[&offer, &None::<gst::Promise>]);


        //let message = serde_json::to_string(&JsonMsg::Sdp
        //    {
        //        type_:  "offer".to_string(),
        //        sdp: offer.sdp().as_text()?,

        //    })
        //.unwrap();
        Ok("meme".to_string())
    }
}
