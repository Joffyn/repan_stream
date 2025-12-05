use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};
use gstreamer::{self as gst, glib::{self, user_name}, message::Element, prelude::{Cast, ElementExt, ElementExtManual, GstBinExtManual, GstObjectExt, PadExt}};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum Request
{
    LoadProject { audiotracks: Vec<String>,
                  projectname: String,
                  user_id: String   },
    SetVolume { audiotrack: String,
                   volume: f64,
                   user_id: String },
    SetPanning {
                audiotrack: String,
                pan: f32,
                user_id: String
                                   },
}
#[derive(Serialize, Deserialize)]
pub struct Response
{
    message: String,
}
 
pub fn disconnect_client(mut stream: &UnixStream)
{
    println!("Client disconnected!");
}

pub fn parse_request(request: Result<Request, serde_json::Error>, manager: Arc<Mutex<pipelinemap::PipelineMap>>) -> Option<Response>
{
    match request
    {
        Ok(Request::LoadProject { audiotracks, projectname, user_id}) =>
        {
            load_project(&audiotracks, &projectname, &user_id, manager);
            Some(Response
            {
                message: format!("Loaded project: '{}'", projectname),
            })
        }
        Ok(Request::SetVolume { audiotrack, volume, user_id }) =>
        {
            let manager = manager.lock().unwrap();
            manager.set_volume(&user_id, &audiotrack, volume);
            Some(Response
            {
                message: format!("Changed volume on track: '{}'", audiotrack),
            })
        }
        Ok(Request::SetPanning { audiotrack, pan, user_id }) =>
        {
            let manager = manager.lock().unwrap();
            manager.set_panning(&user_id, &audiotrack, pan);
            Some(Response
            {
                message: format!("Changed panning on track: '{}'", audiotrack),
            })
        }
        Err(e) =>
        {
            eprintln!("Error with request: {:?}", e);
            Some(Response
            {
                message: format!("Error: '{}'", e),
            })
        }
    }
}
fn load_project(audiotracks: &Vec<String>, projectname: &String, user_id: &String, manager: Arc<Mutex<pipelinemap::PipelineMap>>)
{
    {
        let mut manager = manager.lock().unwrap();

        if manager.user_has_pipeline(user_id)
        {

            let rem_result = manager.remove_pipeline(user_id);
                
            match rem_result 
            {
                Some(pipeline) =>
                {
                    if let Err(e) = pipeline.set_state(gst::State::Null)
                    {
                        eprintln!("Pipeline could not be set to NULL: {:?}", e);
                    }
                    else 
                    {
                        println!("Old pipelien: {} was destroyed", pipeline.name());
                    }
                }
                None => println!("Tried to remove pipeline but user had none"),
            }
            
        }
        else 
        {
            println!("No old pipeline for user: {}", user_id);    
        }
    }

    println!("Loaded project: {}", projectname);
    //Should probably not be done here
    let mut rtmptarget = "rtmp://rtmp:1935/stream/".to_string();
    rtmptarget.push_str(user_id);
    println!("rtmp: {}", rtmptarget);
    let pipeline = audioparser::create_audiopipeline(audiotracks, &rtmptarget);
    pipeline.set_state(gst::State::Playing).expect("Set playstate failed");

    let user_id = user_id.clone();
    let mut manager = manager.lock().unwrap();
    manager.add_pipeline(user_id, pipeline);  
}


