use anyhow::bail;
use gstreamer::{self as gst, Element, Pipeline, glib::{self, SendValue}};
use gst::prelude::*;
use gstreamer_webrtc::{WebRTCSessionDescription, gst::Message};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{self, unbounded_channel};

const STUN_SERVER: &str = "stun://stun.l.google.com:19302";

// JSON messages we communicate with
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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


pub fn connect_user_test()
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

    let (send_ws_msg_tx, send_ws_msg_rx) = mpsc::unbounded_channel::<Message>();

    pipeline.call_async(|pipeline: &Pipeline|
        {
            pipeline
                .set_state(gst::State::Playing)
                .unwrap();
        });

    let _ = webrtcbin
        .connect("on-negotiation-needed", false, move |_| 
            { 
                println!("Negotiations!"); 
                None
            });

    //let _ = webrtcbin
    //    .connect("on-ice-candidate", false, move |_| {})
}

fn on_offer_created(webrtcbin: &gst::Element, reply: Result<&gst::StructureRef, gst::PromiseError>)
    -> Result<(), anyhow::Error>
{

    let reply = match reply
    {
        Ok(reply) => reply,
        Err(e) => bail!("{:?}", e),
    };
    let offer: WebRTCSessionDescription = reply
        .value("offer")
        .unwrap()
        .get::<WebRTCSessionDescription>()
        .unwrap();
    let _ = webrtcbin
        .emit_by_name::<()>("set-local-description", &[&offer, &None::<gst::Promise>]);

    Ok(())

    //let message = serde_json::to_string(&JsonMsg::Sdp 
    //    {
    //        type_:  "offer".to_string(),
    //        sdp: offer.get_sdp().as_text().unwrap(),
    //    })
    //.unwrap();
}



fn on_negotation_needed() -> Result<(), anyhow::Error>
{

}

