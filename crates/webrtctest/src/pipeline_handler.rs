use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::anyhow;
use futures_util::{
    SinkExt, StreamExt,
    stream::{Fuse, SplitSink, SplitStream},
};
use serde::{Deserialize, Serialize};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, accept_async};
use tungstenite::{Message, Utf8Bytes};

use crate::user_connection::UserConn;

#[derive(Debug, Serialize, Deserialize)]
struct ClientMessage {
    gst_msg: GstJsonMsg,
    id: u128,
}

// JSON messages we communicate with
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum GstJsonMsg {
    //Ice {
    //    candidate: String,
    //    #[serde(rename = "sdpMLineIndex")]
    //    sdp_mline_index: u32,
    //},
    Sdp {
        sdp: String,
        #[serde(rename = "type")]
        r#type: String,
    },
}

// JSON messages we communicate with
//#[derive(Serialize, Deserialize)]
//#[serde(rename_all = "lowercase", untagged)]
//enum JsonMsg {
//    Ice {
//        candidate: String,
//        #[serde(rename = "sdpMLineIndex")]
//        sdp_mline_index: u32,
//    },
//    Offer {
//        #[serde(rename = "type")]
//        type_: String,
//        sdp: String,
//    },
//}
pub type RepanStream = Fuse<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>;
pub type RepanSink = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

pub struct Connection {
    //sink: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    //stream: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>
    //stream: Fuse<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>
    stream: RepanStream,
    sink: RepanSink,
    clients: Arc<Mutex<HashMap<u128, UserConn>>>,
}

impl Connection {
    pub async fn new(mut ws: WebSocketStream<MaybeTlsStream<TcpStream>>) -> Self {
        ws.send(Message::text("gstreamer connecting...")).await;
        let (mut sink, mut stream) = ws.split();
        let mut stream = stream.fuse();
        let mut sink = sink;

        println!("Connection to server established");
        Connection {
            sink,
            stream,
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    pub async fn streamer_to_website_handler(&mut self) -> Result<(), anyhow::Error> {
        loop {
            tokio::select! {
                msg = self.stream.select_next_some() =>
                {
                    println!("Message received");
                    if let Ok(msg) = msg
                    {
                        match msg
                        {
                            Message::Text(msg) => self.parse_websocket_msg(msg.to_string()).await? ,
                            Message::Binary(bytes) => todo!(),
                            Message::Ping(bytes) => todo!(),
                            Message::Pong(bytes) => todo!(),
                            Message::Close(close_frame) => todo!(),
                            Message::Frame(frame) => todo!(),
                        }
                    }
                    else
                    {
                        eprintln!("Problem");
                    }
                }
            };
        }
    }
    async fn parse_websocket_msg(&mut self, msg: String) -> Result<(), anyhow::Error> {
        println!("{}", msg.as_str());
        let client_msg: ClientMessage = serde_json::from_str(msg.as_str())?;
        //println!("Was able to parse json");

        //let ClientMessage::Payload { gst_msg, id } = client_msg;
        match client_msg.gst_msg {
            GstJsonMsg::Sdp { r#type, sdp } => {
                let mut user_conn = UserConn::new()?;
                let set_answer = user_conn.set_remote_description(sdp.as_str()).await;
                let sdp_answer = user_conn.create_sdp_answer().await.unwrap();
                let set_desc = user_conn
                    .set_local_description(sdp_answer.clone())
                    .await
                    .unwrap();
                println!("{:?}", set_desc);
                let answer = ClientMessage {
                    gst_msg: GstJsonMsg::Sdp {
                        sdp: sdp_answer,
                        r#type: "answer".to_string(),
                    },
                    id: client_msg.id,
                };
                let answer = serde_json::to_string(&answer).unwrap();
                self.sink.send(Message::Text(Utf8Bytes::from(answer))).await;

                let clients = self.clients.clone();
                let mut guard = clients.lock().await;
                guard.insert(client_msg.id, user_conn);

                //std::thread::sleep(Duration::new(3, 0))

                //println!("{:?}", sdp_answer);
                //println!("{:?}", sdp_answer);

                //let _ = user_conn.deliver_sdp(&sdp.as_str()).unwrap();
                //let _ = self.sink.send(Message::Text(Utf8Bytes::from(sdp_answer))).await?;

                Ok(())
            } //GstJsonMsg::Ice {
              //    sdp_mline_index,
              //    candidate,
              //} => Ok(()),
        }
    }
    async fn handle_user_messages(&self, user_conn: UserConn) {}
}

async fn test(user_conn: UserConn, sink: Arc<RepanSink>) {}
