use crate::backend::{
    client_connections::GstJsonMsg,
    database::JamQueryResult,
    serverfunctions::{get_jam, get_tracks_and_path, MultiQuery},
};
use leptos::{logging::log, prelude::*};
use web_sys::RtcDataChannel;

#[component]
pub fn TrackList(selected_jam_id: ReadSignal<i64>) -> impl IntoView {
    let multi_query = Resource::new(
        move || selected_jam_id.get(),
        move |_| async move { get_jam(selected_jam_id.get()).await },
    );
    let dc = use_context::<ReadSignal<Option<RtcDataChannel>, LocalStorage>>()
        .expect("to have found getter");
    Effect::new(move |_| {
        if let Some(Ok(mq)) = multi_query.get() {
            let path = mq.path.unwrap().data;
            let date = mq.date.unwrap().data;
            let tracks = mq
                .tracks
                .unwrap()
                .iter()
                .map(|v| v.data.clone())
                .collect::<Vec<String>>();
            let json = GstJsonMsg::ChangeJam { path, date, tracks };
            let dc = dc.get().unwrap();
            dc.send_with_str(serde_json::to_string(&json).unwrap().as_str());
        }

        //if let Some(Ok(res)) = tracks_res.get() {
        //    if let Some(paths) = res.get(1).clone() {
        //        if let Some(path_res) = paths.get(0).clone() {
        //            let path = path_res.clone().data;

        //            let tracks = res
        //                .get(0)
        //                .clone()
        //                .unwrap()
        //                .iter()
        //                .map(|v| v.data.clone())
        //                .collect::<Vec<String>>();
        //            //let msg = format! {"Path: {:?}, Tracks: {:?}", path, tracks};
        //            let json = GstJsonMsg::ChangeJam { path, tracks };
        //            //if let Ok(GstJsonMsg::ChangeJam { path, tracks }) =
        //            //    serde_json::from_str(msg.as_str())
        //            //{} // serde_json::json!({path: path, tracks: tracks.as_slice()});
        //            log!("{:?}", json);
        //            let dc = dc.get().unwrap();
        //            dc.send_with_str(serde_json::to_string(&json).unwrap().as_str());
        //        }
        //    }
        //}
    });
    view! {
        <Suspense fallback=move || {
            view! { <p>"Loading..."</p> }
        }>
            {move || Suspend::new(async move {
                let q = multi_query.await;
                let tracks = match q {
                    Ok(q) => q.tracks.unwrap(),
                    Err(_) => {
                        vec![
                            JamQueryResult {
                                id: -1,
                                data: "Null".to_string(),
                            },
                        ]
                    }
                };

                view! {
                    <For each=move || tracks.clone() key=|state| state.id.clone() let(child)>
                        <div>{child.data.clone()}</div>
                    </For>
                }
            })}
        </Suspense>
    }
}
