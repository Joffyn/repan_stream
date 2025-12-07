use gstreamer::{self as gst, Element, Pipeline, glib};
use gst::prelude::*;

pub fn create_audiopipeline(tracks: &Vec<String>, user_id: &str) -> Result<gst::Pipeline, glib::error::BoolError>
{
    let pipeline = gst::Pipeline::with_name(user_id);

    let mixer = gst::ElementFactory::make("audiomixer")
        .build()?;

    pipeline.add(&mixer)?;

    for path in tracks
    {
        create_audiostem(path, &pipeline, &mixer)?;
    }

    //let convert = create_and_link_elements("audioconvert", &mixer, &pipeline);
    //let resample = create_and_link_elements("audioresample", &convert, &pipeline);
    let caps = gst::Caps::builder("audio/x-raw")
        .field("channels", &2)
        .field("rate", &44100)
        .build();

    let capsfilter = gst::ElementFactory::make("capsfilter")
        .property("caps", &caps)
        .build()?;

    pipeline.add(&capsfilter)?;

    let mixer_src = mixer.static_pad("src")?;
   
    let caps_sinkpad = capsfilter.static_pad("sink")?;
    mixer_src.link(&caps_sinkpad)?;

    let encoder = create_and_link_elements("voaacenc", &capsfilter, &pipeline);
    let muxer = gst::ElementFactory::make("flvmux")
        .property("streamable", &true)
        .build()?;

     pipeline.add(&muxer)?;
     let encoder_srcpad = encoder.static_pad("src")?;

     let muxer_sinkpad = muxer.request_pad_simple("audio")?;
     encoder_srcpad.link(&muxer_sinkpad)?;
    

     let webrtc = ElementFactory::make("webrtcbin", Some(&format!("webrtc-{}", user_id)))
         .property("")




     let rtmpsink = gst::ElementFactory::make("rtmpsink")
         .name("rtmpsink")
         .property("location", user_id)
         .build()?;

    pipeline.add(&rtmpsink)?;

    muxer.link(&rtmpsink)?;
    Ok(pipeline)
}

fn create_audiostem(track: &str, pipeline: &gst::Pipeline, mixer: &gst::Element) -> Result<(), glib::error::BoolError>
{
    let track_withprefix = "/audio/".to_string() + track;

    let sound = gst::ElementFactory::make("filesrc")
        .name(track)
        .property("location", track_withprefix)
        .build()?;

    pipeline.add(&sound)?;

    let wavparser = create_and_link_elements("wavparse" , &sound, pipeline)?;
    let audioconvert = create_and_link_elements("audioconvert", &wavparser, pipeline)?;
    let audioresample = create_and_link_elements("audioresample", &audioconvert, pipeline)?;
    let caps = gst::Caps::builder("audio/x-raw")
        .field("channels", &2)
        .field("rate", &44100)
        .build();

    let capsfilter = gst::ElementFactory::make("capsfilter")
        .property("caps", &caps)
        .build()?;

    pipeline.add(&capsfilter)?;
    let resample_src = audioresample.static_pad("src")?;
   
    let caps_sinkpad = capsfilter.static_pad("sink")?;
    resample_src.link(&caps_sinkpad)?;
    //link_elements(&audioresample, &capsfilter);

    let track_volumename = track.to_string() + "_vol";

    let track_volume = gst::ElementFactory::make("volume")
        .name(track_volumename)
        .property("volume", 1.0)
        .property("mute", false)
        .build()?;

    pipeline.add(&track_volume)?;

    let vol_sinkpad = track_volume.static_pad("sink")?;
    let caps_src = capsfilter.static_pad("src")?;
    caps_src.link(&vol_sinkpad)?;

    let def_pan: f32 = 0.0;
    let track_panname = track.to_string() + "_pan";
    let track_panning = gst::ElementFactory::make("audiopanorama")
        .name(track_panname)
        .property("panorama", def_pan)
        .build()?;

    pipeline.add(&track_panning)?;
    //let wavparser_sinkpad = wavparser.static_pad("sink").unwrap();

    //let sound_srcpad = sound.static_pad("src").unwrap();
    //sound_srcpad.link(&wavparser_sinkpad).unwrap();

    //let wavparse_src = wavparser.static_pad("src").unwrap();
    //wavparse_src.link(&vol_sinkpad).unwrap();

    let vol_src = track_volume.static_pad("src")?;
    
    let pan_sinkpad = track_panning.static_pad("sink")?;

    vol_src.link(&pan_sinkpad)?;
    let pan_src = track_panning.static_pad("src")?;

    let mixer_sinkpad = mixer.request_pad_simple("sink_%u")?;
    pan_src.link(&mixer_sinkpad)?;
    // let wavparser = gst::ElementFactory::make("wavparse")
    //     .build()
    //     .expect("couldn't create wavparser");

    // pipeline.add(&wavparser).unwrap();
    Ok(())
}


fn link_elements(src: &gst::Element, sink: &gst::Element) -> Result<(), glib::error::BoolError>
{
    let sink_srcpad = sink.static_pad("src")?;

    let src_sinkpad = src.static_pad("sink")?;
    sink_srcpad.link(&src_sinkpad)?;
    Ok(())
}
fn create_and_link_elements(plugin: &str, sink: &gst::Element, pipeline: &gst::Pipeline) -> Result<gst::Element, glib::error::BoolError>
{
    let newplugin = gst::ElementFactory::make(plugin)
        .build()?;

    pipeline.add(&newplugin)?;

    let sink_srcpad = sink.static_pad("src")?;

    let newplugin_sinkpad = newplugin.static_pad("sink")?;
    sink_srcpad.link(&newplugin_sinkpad)?;
    Ok(newplugin)
}
