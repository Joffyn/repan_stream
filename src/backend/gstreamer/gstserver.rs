use std::sync::{mpsc, Arc, RwLock};
use std::collections::HashMap;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

#[cfg(feature = "ssr")]
use gstreamer::{self as gst, prelude::{ElementExt}};



#[cfg(feature = "ssr")]
#[derive(Clone)]
pub struct PipelineHandle
{
    pub sender: tokio::sync::mpsc::Sender<PipelineCommand>,
}

#[derive(Debug)]
enum PipelineCommand
{
    Play,
    Stop,
    SetPanning(f64),
    SetVolume(f64),
    Disconnect,
    Pause,
    Shutdown,
}

#[cfg(feature = "ssr")]
type Pipelines = Arc<RwLock<HashMap<String, PipelineHandle>>>;

#[cfg(feature = "ssr")]
pub fn spawn_pipeline(user_id: String, pipelines: Pipelines)
{
    let pipeline = gst::Pipeline::with_name(user_id.as_str());

    let (tx, rx) = tokio::sync::mpsc::channel(8);

    {
        let mut map = pipelines.write().unwrap();
        map.insert(user_id.clone(), PipelineHandle { sender: tx.clone() });
        //map.add_pipeline(user_id.clone(), pipeline.clone());
    }
        tokio::spawn(async move {
        pipeline_task(user_id, pipeline, rx).await;
    });

}
#[cfg(feature = "ssr")]
async fn pipeline_task(user_id: String, pipeline: gst::Pipeline, mut rx: tokio::sync::mpsc::Receiver<PipelineCommand>) {
    println!("[{user_id}] Pipeline task started");

    // Start paused
    pipeline.set_state(gst::State::Paused).unwrap();

    // Get the pipeline's bus
    let bus = pipeline.bus().unwrap();

    loop {
        tokio::select! {
            Some(cmd) = rx.recv() => {
                match cmd {
                    PipelineCommand::Play => {
                        println!("[{user_id}] Play");
                        pipeline.set_state(gst::State::Playing).unwrap();
                    }
                    PipelineCommand::Pause => {
                        println!("[{user_id}] Pause");
                        pipeline.set_state(gst::State::Paused).unwrap();
                    }
                    PipelineCommand::Stop => {
                        println!("[{user_id}] Stop");
                        pipeline.set_state(gst::State::Null).unwrap();
                    }
                    PipelineCommand::Shutdown => {
                        println!("[{user_id}] Shutdown");
                        pipeline.set_state(gst::State::Null).unwrap();
                        break;
                    }
                    _ => println!("[{user_id}] No Req"),
                }
            }

            // Poll GStreamer bus for messages
            //msg = bus.timed_pop(gst::ClockTime::from_seconds(1)) => {
            //    if let Some(msg) = msg {
            //        use gst::MessageView;
            //        match msg.view() {
            //            MessageView::Eos(..) => {
            //                println!("[{user_id}] Reached EOS");
            //                pipeline.set_state(gst::State::Null).unwrap();
            //                break;
            //            }
            //            MessageView::Error(err) => {
            //                eprintln!(
            //                    "[{user_id}] Error from {:?}: {} ({:?})",
            //                    err.src().map(|s| s.path_string()),
            //                    err.error(),
            //                    err.debug()
            //                );
            //                pipeline.set_state(gst::State::Null).unwrap();
            //                break;
            //            }
            //            _ => {}
            //        }
            //    }
            //}
        }
    }

    println!("[{user_id}] Pipeline task exited");
}

// Create a new pipeline for a user
#[server]
pub async fn create_pipeline(user_id: String) -> Result<(), ServerFnError> {
    let pipelines = use_context::<Arc<RwLock<HashMap<String, PipelineHandle>>>>().unwrap();

    spawn_pipeline(user_id, pipelines);
    Ok(())
}

// Send a play command
#[server]
pub async fn play_pipeline(user_id: String) -> Result<(), ServerFnError> {
    let pipelines = use_context::<Arc<RwLock<HashMap<String, PipelineHandle>>>>().unwrap();

    // Clone out the handle while the lock is held
    let handle = {
        let map = pipelines.read().unwrap();
        map.get(&user_id).cloned()
    };
    if let Some(handle) = handle {
        handle.sender.send(PipelineCommand::Play).await.unwrap();
        Ok(())
    } else {
        Err(ServerFnError::ServerError("Pipeline not found".into()))
    }

}
