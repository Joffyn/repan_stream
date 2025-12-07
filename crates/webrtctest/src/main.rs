use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures_util::{StreamExt, SinkExt};
use tungstenite::Message;


mod webrtc_conn;

#[tokio::main]
async fn main() -> anyhow::Result<()>
{
    let listener = TcpListener::bind("127.0.0.1:8443").await?;
    while let Ok((stream, _)) = listener.accept().await
    {
        tokio::spawn(async move  
        {
            let ws = accept_async(stream).await.unwrap();
            let (mut w, mut r) = ws.split();
            while let Some(Ok(msg)) = r.next().await
            {
                let ret = Message::text("Back to you");
                let _ = w.send(ret).await;
            }
        });
    }
    Ok(())
}

