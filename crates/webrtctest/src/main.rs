#![allow(dead_code)]
#![allow(unused)]
use anyhow::anyhow;
use rand::Rng;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, accept_async};
use futures_util::{SinkExt, StreamExt, stream::SplitStream};
use tungstenite::Message;

use crate::pipeline_handler::Connection;

//use crate::{webrtc_conn::UserConn};




mod user_connection;
//mod webrtc_conn;
mod pipeline_handler;
#[tokio::main]
async fn main() -> Result<(), anyhow::Error>
{
    gstreamer::init().unwrap();
    
    let (mut ws, response) = tokio_tungstenite::connect_async("ws://127.0.0.1:3000/ws").await?;
    let mut conn = Connection::new(ws).await;
    conn.streamer_to_website_handler().await;


    Ok(())
}

//#[tokio::main]
//async fn main() -> Result<(), anyhow::Error>
//{
//    gstreamer::init().unwrap();
//    let (pipeline, pipeline_rx, ws_rx) = UserConn::new().unwrap();
//    let mut pipeline_rx = pipeline_rx.fuse();
//    let mut ws_rx = ws_rx.fuse();
//
//    let mut ws = tokio_tungstenite::connect_async("ws://127.0.0.1:3000/ws").await;
//
//    let bytes = vec![0;1];
//    
//    if let Ok((mut socket, response)) = ws 
//    {
//        println!("Connected");
//        let id = rand::rng().random_range(10..1000);
//        //socket.send(Message::binary(bytes)).await;
//        //socket.send(Message::text(format!("Hello from {}!", id))).await;
//        socket.send(Message::text("gstreamer connecting...")).await;
//        //println!("1");
//        //let msg = socket.select_next_some().await.unwrap();
//        //println!("2");
//
//        //println!("{:?}", msg.unwrap().to_text().unwrap());
//        let (mut ws_sink, ws_stream) = socket.split();
//        let mut ws_stream = ws_stream.fuse();
//
//        //tokio::spawn(test(ws_stream));
//
//        loop
//        {
//            let ws_msg = tokio::select! 
//            {
//                ws_msg = ws_stream.select_next_some() =>
//                {
//                    println!("New message");
//                    if let Ok(msg) = ws_msg
//                    {
//                        if msg.is_text()
//                        {
//                            pipeline.handle_websocket_message(msg.to_text().unwrap())?;
//                        }
//                        else if msg.is_close()
//                        {
//
//                            println!("peer disconnected");
//                            break
//                        }
//                        None
//                    }
//                    else
//                    {
//                        None
//                    }
//                    //match ws_msg?
//                    //{
//                    //    Message::Close(_) => 
//                    //    {
//                    //        println!("peer disconnected");
//                    //        break
//                    //    },
//                    //    Message::Ping(data) => Some(Message::Pong(data)),
//                    //    Message::Pong(_) => None,
//                    //    Message::Binary(_) => None,
//                    //    Message::Text(text) => 
//                    //    {
//                    //        pipeline.handle_websocket_message(&text)?;
//                    //        None
//                    //    },
//                    //    _ => None,
//                    //}
//                },
//                gst_msg = pipeline_rx.select_next_some() =>
//                {
//                    pipeline.handle_pipeline_message(&gst_msg);
//                    None
//                },
//
//                ws_msg = ws_rx.select_next_some() => Some(ws_msg),
//
//            };
//            if let Some(ws_msg) = ws_msg
//            {
//                ws_sink.send(ws_msg).await?;
//            }
//        }
//    }
//    else 
//    {
//        eprintln!("No server to connect to!");  
//    }
//    Ok(())
//}
async fn test(mut ws_stream: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>)
{
    loop
    {
        while let Some(Ok(msg)) = ws_stream.next().await
        {
            println!("{}", msg.to_text().unwrap());
        }
    }
}
