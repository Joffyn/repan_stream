#![allow(dead_code)]
#![allow(unused)]
use anyhow::anyhow;
use rand::Rng;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures_util::{StreamExt, SinkExt};
use tungstenite::Message;

use crate::webrtc_conn::UserConn;


mod webrtc_conn;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error>
{
    gstreamer::init().unwrap();
    let (pipeline, pipeline_rx, ws_rx) = UserConn::new().unwrap();
    let mut pipeline_rx = pipeline_rx.fuse();
    let mut ws_rx = ws_rx.fuse();

    let mut ws = tokio_tungstenite::connect_async("ws://127.0.0.1:8443").await;

    let bytes = vec![0;1];
    
    if let Ok((mut socket, _)) = ws 
    {
        println!("Connected");
        let id = rand::rng().random_range(10..1000);
        //socket.send(Message::binary(bytes)).await;
        //socket.send(Message::text(format!("Hello from {}!", id))).await;
        socket.send(Message::text("gstreamer")).await;

        let msg = socket.next().await.unwrap();

        println!("{:?}", msg.unwrap().to_text().unwrap());
        let (mut ws_sink, ws_stream) = socket.split();
        let mut ws_stream = ws_stream.fuse();

        loop
        {
            let ws_msg = tokio::select! 
            {
                ws_msg = ws_stream.select_next_some() =>
                {
                    match ws_msg?
                    {
                        Message::Close(_) => 
                        {
                            println!("peer disconnected");
                            break
                        },
                        Message::Ping(data) => Some(Message::Pong(data)),
                        Message::Pong(_) => None,
                        Message::Binary(_) => None,
                        Message::Text(text) => 
                        {
                            pipeline.handle_websocket_message(&text)?;
                            None
                        },
                        _ => None,
                    }
                },
                gst_msg = pipeline_rx.select_next_some() =>
                {
                    pipeline.handle_pipeline_message(&gst_msg);
                    None
                },

                ws_msg = ws_rx.select_next_some() => Some(ws_msg),

            };
            if let Some(ws_msg) = ws_msg
            {
                ws_sink.send(ws_msg).await?;
            }
        }
        //while let Some(Ok(msg)) = socket.next().await
        //{

        //    match msg
        //    {
        //        //Message::Close(_) => 
        //        //{
        //        //    println!("peer disconnected");
        //        //    break;
        //        //},
        //        //Message::Ping(data) => Message::Pong(data),
        //        //Message::Pong(_) => None,
        //        //Message::Binary(_) => None,
        //        Message::Text(text) => 
        //        {
        //            pipeline.handle_websocket_message(&text).unwrap();
        //        },
        //        _ => (),
        //    }
        //}
    }
    else 
    {
        eprintln!("No server to connect to!");  
    }
    Ok(())
}
