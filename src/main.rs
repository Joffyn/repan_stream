#![allow(warnings)]
#![warn(unused_variables)]
use repan_stream::app::frontend::pipelinemap::PipelineMap;
use leptos::config::LeptosOptions;
use std::sync::{Arc, RwLock};


#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() 
{

    use std::collections::HashMap;

    use axum::Router;
    use leptos::{logging::log, prelude::provide_context};
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};

    use repan_stream::app::*;
    use repan_stream::app::gstmod::gstserver::*;
    use gstreamer::{self as gst, glib::property::PropertyGet, prelude::{GstBinExtManual, GstObjectExt, ObjectExt}};
    
    gst::init().expect("to initialize gstreamer");
    

    let pipelines = Arc::new(RwLock::new(HashMap::<String, PipelineHandle>::new()));

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);



    let app = Router::new()
        .leptos_routes_with_context(&leptos_options, routes, move || provide_context(pipelines.clone()) ,{
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

    log!("Gstreamer pipeline started");
}
fn main() 
{

}
