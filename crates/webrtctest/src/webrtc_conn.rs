use std::sync::{Arc, Weak};

use anyhow::{Context, bail};
use async_tungstenite::tungstenite;
use gstreamer::{self as gst, Pipeline, StructureRef};
use gst::prelude::*;
use gstreamer_webrtc::{WebRTCSessionDescription, gst::Message};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

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

#[derive(Debug, Clone)]
struct UserConn(Arc<UserConnectionInner>);

#[derive(Debug, Clone)]
struct WeakConn(Weak<UserConnectionInner>);

#[derive(Debug)]
struct UserConnectionInner
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


    fn new() -> Result<Self, anyhow::Error>
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
                });

        let conn = UserConn(Arc::new(UserConnectionInner 
                {
                    pipeline,
                    webrtcbin,
                    send_msg_tx: send_ws_msg_tx
                }));


        let conn_clone = conn.downgrade();

        let _ = conn.webrtcbin
            .connect("on-negotiation-needed", false, move |values|
                {
                    let _webrtc = values[0].get::<gst::Element>().unwrap();
                    let conn = upgrade_weak!(conn_clone, None);
                    if let Err(e) = conn.on_negotation_needed()
                    {
                        eprintln!("{:?}", e);
                    }
                    None
                });


        let conn_clone = conn.downgrade();

        let _ = conn.webrtcbin
            .connect("on-ice-candidate", false, move |values|
                {
                    let _webrtc = values[0].get::<gst::Element>().unwrap();
                    let mlineindex = values[1].get::<u32>().unwrap();
                    let candidate = values[2]
                        .get::<String>()
                        .unwrap();

                    let conn = upgrade_weak!(conn_clone, None);
                    if let Err(e) = conn.on_ice_candidate(mlineindex, candidate)
                    {
                        eprintln!("{:?}", e);
                    }
                    None
                });

        let conn_clone = conn.downgrade();

        let _ = conn.webrtcbin.connect_pad_added(move |_webrtc, pad|
            {
                let conn = upgrade_weak!(conn_clone);

                if let Err(e) = conn.on_incoming_stream(pad)
                {
                    eprintln!("{:?}", e);
                }
            });


        conn.pipeline.call_async(|pipeline| 
            {
                if pipeline.set_state(gst::State::Playing).is_err()
                {
                    eprintln!("Failure to set pipeline to playing");
                }

            });
        Ok(conn)
    }


    fn on_negotation_needed(&self) -> Result<(), anyhow::Error>
    {
        println!("starting negotiation");

        let conn_clone = self.downgrade();
        let promise = gst::Promise::with_change_func(move |reply| 
        {
            let conn = upgrade_weak!(conn_clone);

            if let Err(e) = conn.on_offer_created(reply) 
            {
                eprintln!("{:?}", e);
            }
        });

        self.webrtcbin
            .emit_by_name::<()>("create-offer", &[&None::<gst::Structure>, &promise]);

        Ok(())    
    }

    fn on_offer_created(&self,reply: Result<Option<&StructureRef>, gst::PromiseError>) -> Result<(), anyhow::Error>
    {


        let reply = match reply
        {
            Ok(reply) => reply,
            Err(e) => bail!("{:?}", e),
        };
        let offer: WebRTCSessionDescription = reply
            .unwrap()
            .value("offer")
            .unwrap()
            .get::<WebRTCSessionDescription>()
            .unwrap();

        println!(
            "sending SDP offer to peer: {}",
            offer.sdp().as_text().unwrap()
        );

        self.webrtcbin
            .emit_by_name::<()>("set-local-description", &[&offer, &None::<gst::Promise>]);


        let message = serde_json::to_string(&JsonMsg::Sdp
            {
                type_:  "offer".to_string(),
                sdp: offer.sdp().as_text()?,

            })
        .unwrap();

        self.send_msg_tx.send(tungstenite::Message::text(message)).with_context(|| format!("Failed to send SDP offer"))?;
        Ok(())
    }
    fn on_ice_candidate(&self, mlineindex: u32, candidate: String) -> Result<(), anyhow::Error>
    {
        let message = serde_json::to_string(&JsonMsg::Ice {
            candidate,
            sdp_mline_index: mlineindex,
        })
        .unwrap();


        self.send_msg_tx.send(tungstenite::Message::text(message)).with_context(|| format!("Failed to send ICE candidate"))?;
        Ok(())    
    }
    fn on_incoming_stream(&self, pad: &gst::Pad) -> Result<(), anyhow::Error>
    {
        // Early return for the source pads we're adding ourselves
        if pad.direction() != gst::PadDirection::Src {
            return Ok(());
        }

        let decodebin = gst::ElementFactory::make("decodebin")
            .build()
            .unwrap();
        let app_clone = self.downgrade();
        decodebin.connect_pad_added(move |_decodebin, pad| 
        {
            let app = upgrade_weak!(app_clone);

            if let Err(e) = app.on_incoming_decodebin_stream(pad) 
            {
                eprintln!("{:?}", e);
            }
        });

        self.pipeline.add(&decodebin).unwrap();
        decodebin.sync_state_with_parent().unwrap();

        let sinkpad = decodebin.static_pad("sink").unwrap();
        pad.link(&sinkpad).unwrap();

        Ok(())
    }
    fn on_incoming_decodebin_stream(&self, pad: &gst::Pad) -> Result<(), anyhow::Error> 
    {
        let caps = pad.current_caps().unwrap();
        let name = caps.structure(0).unwrap().name();

        let sink = if name.starts_with("audio/") 
        {
            gst::parse::bin_from_description(
                "queue ! audioconvert ! audioresample ! autoaudiosink",
                true,
            )?
        } 
        else 
        {
            println!("Unknown pad {:?}, ignoring", pad);
            return Ok(());
        };

        self.pipeline.add(&sink).unwrap();
        sink.sync_state_with_parent()
            .with_context(|| format!("can't start sink for stream {:?}", caps))?;

        let sinkpad = sink.static_pad("sink").unwrap();
        pad.link(&sinkpad)
            .with_context(|| format!("can't link sink for stream {:?}", caps))?;

        Ok(())
    }
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


    let message = serde_json::to_string(&JsonMsg::Sdp
        {
            type_:  "offer".to_string(),
            sdp: offer.sdp().as_text()?,

        })
    .unwrap();


    Ok(())
}



//fn on_negotation_needed() -> Result<(), anyhow::Error>
//{
//
//}

