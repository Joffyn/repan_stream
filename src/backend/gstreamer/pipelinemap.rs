use std::collections::HashMap;
#[cfg(feature = "ssr")]
use gstreamer::{self as gst, glib::{self, property::PropertyGet, user_name}, message::Element, prelude::{Cast, ElementExt, ElementExtManual, GstBinExtManual, GstObjectExt, ObjectExt, PadExt}};



#[cfg(feature = "ssr")]
#[derive(Clone)]
pub struct PipelineMap
{
    pipelines: HashMap<String, gst::Pipeline>,
}
#[cfg(feature = "ssr")]
impl PipelineMap
{
#[cfg(feature = "ssr")]
    pub fn new() -> Self
    {
        PipelineMap
        {
            pipelines: HashMap::new(),
        }
    }

#[cfg(feature = "ssr")]
    pub fn add_pipeline(&mut self, user_id: String, pipeline: gst::Pipeline)
    {
        self.pipelines.insert(user_id, pipeline);
    }
    pub fn get_pipeline(&self, user_id: &String) -> &gst::Pipeline
    {
        return self.pipelines.get(user_id).expect("User has no pipeline");
    }
#[cfg(feature = "ssr")]
    pub fn user_has_pipeline(&self, user_id: &String) -> bool
    {
        return self.pipelines.contains_key(user_id);
    }
#[cfg(feature = "ssr")]
    pub fn remove_pipeline(&mut self, user_id: &String) -> Option<gst::Pipeline>
    {
        return self.pipelines.remove(user_id);
    }
#[cfg(feature = "ssr")]
    pub fn set_panning(&self, user_id: &String, track_name: &String, pan: f32)
    {
        let pipeline = self.pipelines.get(user_id).expect("Tried to set panning but user has no pipeline");

        let track_panname = track_name.clone() + "_pan";
        println!("Setting panning on: {}, pipeline: {}, pan: {}", track_panname, pipeline.name(), pan.to_string());
        let elem = get_element(pipeline, &track_panname);

        match elem 
        {
            Some(elem) =>
            {
                elem.set_property("panorama", pan);
                println!("Set panning on track: {} to {}", track_name, pan.to_string());
            },
            None => 
            {
                eprintln!("Tried to set panning but track does not exist");
            }
        }
    }
#[cfg(feature = "ssr")]
    pub fn set_volume(&self, user_id: &String, track_name: &String, volume: f64)
    {
        let pipeline = self.pipelines.get(user_id).expect("Tried to set volume but user has no pipeline");


        let track_volname = track_name.clone() + "_vol";
        println!("Setting volume on: {}, pipeline: {}, volume: {}", track_volname, pipeline.name(), volume.to_string());
        let elem = get_element(pipeline, &track_volname);

        match elem 
        {
            Some(elem) =>
            {
                elem.set_property("volume", volume);
                println!("Set volume on track: {} to {}", track_name, volume.to_string());
            },
            None => 
            {
                eprintln!("Tried to set volume but track does not exist");
            }
        }
    }
    #[cfg(feature = "ssr")]
    pub fn set_pipeline_state(&self, user_id: &String, new_state: gst::State)
    {
        let pipeline = self.pipelines.get(user_id).expect("Tried to set playstate but user has no pipeline");

        pipeline.set_state(new_state).expect("To change playstate");
    }
}

#[cfg(feature = "ssr")]
fn get_element(pipeline: &gst::Pipeline , name: &String) -> Option<gst::Element>
{
    for elem in pipeline.iterate_elements()
    {
        match elem
        {
            Ok(elem) => 
            {
                println!("Iterating over elements: {}", elem.name());
                println!("Trackname: {}", *name);
                if elem.name().to_string() == *name
                {
                    return Some(elem);
                }
            }
            Err(e) => eprintln!("{}", e),
        }
    }
    println!("No element found");
    return None;
}
