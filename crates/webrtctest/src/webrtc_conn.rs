use std::sync::{Arc, Weak};

use anyhow::{Context, anyhow, bail};
use async_tungstenite::tungstenite;
use futures_util::{Stream, stream::futures_unordered};
use gstreamer::{self as gst, Pipeline, StructureRef, glib::GString};
use gst::prelude::*;
use gstreamer_webrtc::{WebRTCSDPType, WebRTCSessionDescription, gst::Message, gst_sdp};
use serde::{Deserialize, Serialize};
use tokio::{sync::{futures, mpsc::{self, UnboundedReceiver}}};
use tokio_stream::wrappers::UnboundedReceiverStream;

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


    pub fn new() -> Result<(Self, impl Stream<Item = gstreamer::Message>, impl Stream<Item = tungstenite::Message>), anyhow::Error>
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
        Ok((conn, send_gst_msg_rx, UnboundedReceiverStream::new(send_ws_msg_rx)))
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

        //println!(
        //    "sending SDP offer to peer: {}",
        //    offer.sdp().as_text().unwrap()
        //);

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

    // Once webrtcbin has create the answer SDP for us, handle it by sending it to the peer via the
    // WebSocket connection
    fn on_answer_created(
        &self,
        reply: Result<Option<&StructureRef>, gst::PromiseError>,
    ) -> Result<(), anyhow::Error> 
    {
        let reply = match reply 
        {
            Ok(reply) => reply,
            Err(err) => 
            {
                bail!("Answer creation future got no reponse: {:?}", err);
            }
        };

        let answer = reply
            .unwrap()
            .value("answer")
            .unwrap()
            .get::<WebRTCSessionDescription>()
            .unwrap();

        self.webrtcbin
            .emit_by_name::<()>("set-local-description", &[&answer, &None::<gst::Promise>]);

        println!(
            "sending SDP answer to peer: {}",
            answer.sdp().as_text().unwrap()
        );

        let message = serde_json::to_string(&JsonMsg::Sdp {
            type_: "answer".to_string(),
            sdp: answer.sdp().as_text().unwrap(),
        })
        .unwrap();

        self.send_msg_tx.send(tungstenite::Message::text(message)).with_context(|| format!("Failed to send SDP answer"))?;

        Ok(())
    }
    pub fn handle_websocket_message(&self, msg: &str) -> Result<(), anyhow::Error>
    {
        if msg.starts_with("ERROR") 
        {
            bail!("Got error message: {}", msg);
        }

        println!("{}", msg);
        let json_msg: JsonMsg = serde_json::from_str(msg)?;
        println!("Parsed as Json");

        match json_msg 
        {
            JsonMsg::Sdp { type_, sdp } => self.handle_sdp(&type_, &sdp),
            JsonMsg::Ice 
            {
                sdp_mline_index,
                candidate,
            } => self.handle_ice(sdp_mline_index, &candidate),
        }
    }
    // Handle GStreamer messages coming from the pipeline
    pub fn handle_pipeline_message(&self, message: &gst::Message) -> Result<(), anyhow::Error> 
    {
        use gst::message::MessageView;

        match message.view() 
        {
            MessageView::Error(err) => bail!(
                "Error from element {}: {} ({})",
                err.src()
                    .map(|s| String::from(s.path_string()))
                    .unwrap_or_else(|| String::from("None")),
                err.error(),
                err.debug().unwrap_or_else(|| GString::from("None")),
            ),
            MessageView::Warning(warning) => 
            {
                println!("Warning: \"{}\"", warning.debug().unwrap());
            }
            _ => (),
        }

        Ok(())
    }
    fn handle_sdp(&self, type_: &str, sdp: &str) -> Result<(), anyhow::Error> {
        if type_ == "answer" 
        {
            print!("Received answer:\n{}\n", sdp);

            let ret = gst_sdp::SDPMessage::parse_buffer(sdp.as_bytes())
                .map_err(|_| anyhow!("Failed to parse SDP answer"))?;
            let answer =
                WebRTCSessionDescription::new(WebRTCSDPType::Answer, ret);

            self.webrtcbin
                .emit_by_name::<()>("set-remote-description", &[&answer, &None::<gst::Promise>]);

            Ok(())
        } 
        else if type_ == "offer" 
        {
            print!("Received offer:\n{}\n", sdp);

            let ret = gst_sdp::SDPMessage::parse_buffer(sdp.as_bytes())
                .map_err(|_| anyhow!("Failed to parse SDP offer"))?;

            // And then asynchronously start our pipeline and do the next steps. The
            // pipeline needs to be started before we can create an answer
            let app_clone = self.downgrade();
            self.pipeline.call_async(move |_pipeline| {
                let app = upgrade_weak!(app_clone);

                let offer = WebRTCSessionDescription::new(
                    WebRTCSDPType::Offer,
                    ret,
                );

                app.0
                    .webrtcbin
                    .emit_by_name::<()>("set-remote-description", &[&offer, &None::<gst::Promise>]);

                let app_clone = app.downgrade();
                let promise = gst::Promise::with_change_func(move |reply| 
                {
                    let app = upgrade_weak!(app_clone);

                    if let Err(e) = app.on_answer_created(reply) 
                    {
                        eprintln!("{:?}", e);
                    }
                });

                app.0
                    .webrtcbin
                    .emit_by_name::<()>("create-answer", &[&None::<gst::Structure>, &promise]);
                });

            Ok(())
        } 
        else 
        {
            bail!("Sdp type is not \"answer\" but \"{}\"", type_)
        }
    }

    // Handle incoming ICE candidates from the peer by passing them to webrtcbin
    fn handle_ice(&self, sdp_mline_index: u32, candidate: &str) -> Result<(), anyhow::Error> 
    {
        self.webrtcbin
            .emit_by_name::<()>("add-ice-candidate", &[&sdp_mline_index, &candidate]);

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

