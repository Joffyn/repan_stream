#![allow(dead_code)]
#![allow(unused)]
use tokio::net::TcpListener;
use tokio_tungstenite::{WebSocketStream, accept_async};
use futures_util::{StreamExt, SinkExt};
use tungstenite::{Message, Utf8Bytes, accept, protocol::{CloseFrame, frame::coding::CloseCode}};



#[tokio::main]
async fn main() -> anyhow::Result<()>
{
    let listener = TcpListener::bind("127.0.0.1:8443").await?;
    while let Ok((stream, addr)) = listener.accept().await
    {
        println!("Accepted new user: {}!", addr);

        let ws = accept_async(stream).await.unwrap();
        let (mut w, mut r) = ws.split();
        if let Some(Ok(id)) = r.next().await
        {
            if !id.is_text()
            {
                w.reunite(r)
                    .unwrap()
                    .close(Some(CloseFrame { code: CloseCode::Invalid, reason: Utf8Bytes::from_static("Need to send ID first")}))
                    .await;
                    
            }
            else if let Ok(id) = id.to_text()
            {
                if id.eq("gstreamer")
                {
                    tokio::spawn(handle_gstreamer(w.reunite(r).unwrap()));
                }
                else
                {
                    tokio::spawn(handle_user(w.reunite(r).unwrap()));
                }

            }
        }


        //tokio::spawn(async move  
        //{
        //    let ws = accept_async(stream).await.unwrap();
        //    let (mut w, mut r) = ws.split();
        //    while let Some(Ok(msg)) = r.next().await
        //    {
        //        println!("{}", msg);
        //        let ret = Message::text("Back to you");
        //        let _ = w.send(ret).await;
        //    }
        //});
    }
    Ok(())
}
async fn handle_gstreamer(ws: WebSocketStream<tokio::net::TcpStream>)
{
    let (mut w, mut r) = ws.split();
    w.send(Message::text("Welcome Gstreamer")).await;
    while let Some(Ok(msg)) = r.next().await
    {
        println!("{}", msg);
        let ret = Message::text("back to gstreamer");
        let _ = w.send(ret).await;
    }
}

async fn handle_user(ws: WebSocketStream<tokio::net::TcpStream>)
{
    let (mut w, mut r) = ws.split();
    w.send(Message::text("Welcome User")).await;
    while let Some(Ok(msg)) = r.next().await
    {
        println!("{}", msg);
        let ret = Message::text("back to you");
        let _ = w.send(ret).await;
    }
}
