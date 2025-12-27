#![allow(warnings)]
#![warn(unused_variables)]
use axum::{
    extract::{
        ws::{Message, WebSocket},
        WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use leptos::{config::LeptosOptions, prelude::guards};
use leptos::{logging::log, prelude::provide_context};
use repan_stream::backend::{
    client_connections::{self, ws_handler, CLIENT_SDP_ANSWERS},
    database::Database,
};
use tokio::{
    stream,
    sync::mpsc::{channel, Receiver, Sender},
};

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use std::collections::HashMap;

    use axum::routing::{any, get};
    use axum::Router;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};

    use repan_stream::backend::database::get_database;
    use repan_stream::{app::*, backend};

    match get_database() {
        Err(e) => {
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
