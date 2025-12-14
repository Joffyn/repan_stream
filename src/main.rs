#![allow(warnings)]
#![warn(unused_variables)]
use axum::{extract::{WebSocketUpgrade, ws::{Message, WebSocket}}, response::IntoResponse};
use futures::{SinkExt, StreamExt, stream::{SplitSink, SplitStream}};
use leptos::config::LeptosOptions;
use repan_stream::{backend::database::Database, frontend::webrtc::GSTREAMER_SENDER};
use leptos::{logging::log, prelude::provide_context};
use tokio::{stream, sync::mpsc::{Receiver, Sender, channel}};


#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() 
{
    use std::collections::HashMap;

    use axum::Router;
    use axum::routing::{any, get};
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};

    use repan_stream::backend::database::get_database;
    use repan_stream::{app::*, backend};

    match get_database()
    {
        Err(e) => 
        {
            eprintln!("{:?}", e);
            return; 
        }
        _ => println!("Database loaded"),
    }
    
    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);



    let app = Router::new()
        .route("/ws", any(ws_handler))
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options.clone());

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();

}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse
{
    log!("Connected!");
    ws.on_failed_upgrade(|e| eprintln!("{:?}", e))
        .on_upgrade(handle_socket)
}
async fn handle_socket(mut socket: WebSocket)
{
    use axum::extract::ws::Utf8Bytes;

    socket.send(Message::Text(Utf8Bytes::from_static("Hello!")));
    log!("Socket being handled");
    let (mut client_tx, mut client_rx) = tokio::sync::mpsc::channel::<String>(128);

    let (mut streamer_sender, mut streamer_receiver) = socket.split();
    let client_to_streamer =  GSTREAMER_SENDER.clone();
    *client_to_streamer.write().await = Some(client_tx);

    tokio::spawn(handle_client_messages(streamer_sender, client_rx));
    tokio::spawn(handle_streamer_messages(streamer_receiver));
    //loop
    //{

    //    match client_rx.recv().await
    //    {
    //        Some(msg) =>
    //        {
    //            log!("Sent message from server!");
    //            streamer_sender.send(Message::Text(Utf8Bytes::from(msg)));
    //        },
    //        _ => (),
    //    }
    //    match streamer_receiver.next().await
    //    {
    //        Some(Ok(msg)) => log!("{:?}", msg),
    //        Some(Err(e)) => log!("{:?}", e),
    //        None => 
    //        {
    //            log!("Socket ended");
    //            break;
    //        }
    //    }

    //}
    log!("Socket disconnecting");
}
async fn handle_client_messages(mut tx: SplitSink<WebSocket, Message>, mut rx: Receiver<String>)
{
    use axum::extract::ws::Utf8Bytes;
    loop
    {
        match rx.recv().await
        {
            Some(msg) =>
            {
                log!("Sent message from server!");
                let _ = tx.send(Message::Text(Utf8Bytes::from(msg))).await;
            },
            _ => (),
        }
    }
}
async fn handle_streamer_messages(mut rx: SplitStream<WebSocket>)
{
    loop
    {
        match rx.next().await
        {
            Some(Ok(msg)) => log!("{:?}", msg.to_text().unwrap()),
            Some(Err(e)) => log!("{:?}", e.to_string()),
            None => 
            {
                log!("Socket ended");
                break;
            }
        }
    }
}
