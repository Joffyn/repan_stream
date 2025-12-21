use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::anyhow;
use futures_util::{
    SinkExt, StreamExt,
    stream::{Fuse, SplitSink, SplitStream},
};
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, accept_async};
use tungstenite::{Message, Utf8Bytes};

use crate::user_connection::UserConn;

// JSON messages we communicate with
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase", untagged)]
enum JsonMsg {
    Ice {
        candidate: String,
        #[serde(rename = "sdpMLineIndex")]
        sdp_mline_index: u32,
    },
    Offer {
        #[serde(rename = "type")]
        type_: String,
        sdp: String,
    },
}
pub type RepanStream = Fuse<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>;
pub type RepanSink = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

pub struct Connection {
    //sink: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    //stream: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>
    //stream: Fuse<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>
    stream: RepanStream,
    sink: RepanSink,
    clients: HashMap<u32, UserConn>,
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
            clients: HashMap::new(),
        }
    }
    pub async fn streamer_to_website_handler(&mut self) -> Result<(), anyhow::Error> {
        loop {
            tokio::select! {
                msg = self.stream.select_next_some() =>
                {
                    if let Ok(msg) = msg
                    {
                        match msg
                        {
                            Message::Text(msg) => self.parse_websocket_msg(msg.as_str()).await? ,
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
    async fn parse_websocket_msg(&mut self, msg: &str) -> Result<(), anyhow::Error> {
        let json_msg: JsonMsg = serde_json::from_str(msg)?;
        println!("Parsed as Json");

        match json_msg {
            JsonMsg::Offer { type_, sdp } => {
                let mut user_conn = UserConn::new()?;
                let set_answer = user_conn.set_remote_description(sdp.as_str()).await;
                let sdp_answer = user_conn.create_sdp_answer().await.unwrap();
                //std::thread::sleep(Duration::new(3, 0))

                //println!("{:?}", sdp_answer);
                println!("{:?}", sdp_answer);

                //let _ = user_conn.deliver_sdp(&sdp.as_str()).unwrap();
                //let _ = self.sink.send(Message::Text(Utf8Bytes::from(sdp_answer))).await?;

                Ok(())
            }
            JsonMsg::Ice {
                sdp_mline_index,
                candidate,
            } => Ok(()),
        }
    }
    async fn handle_user_messages(&self, user_conn: UserConn) {}
}

async fn test(user_conn: UserConn, sink: Arc<RepanSink>) {}
