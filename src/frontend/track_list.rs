use crate::backend::{database::JamQueryResult, serverfunctions::get_track_list};
use leptos::prelude::*;

#[component]
pub fn TrackList(selected_jam_id: ReadSignal<i64>) -> impl IntoView {
    let tracks_res = Resource::new(
        move || selected_jam_id.get(),
        move |_| async move { get_track_list(selected_jam_id.get()).await },
    );
    view! {
        <Suspense fallback=move || {
            view! { <p>"Loading..."</p> }
        }>
            {move || Suspend::new(async move {
                let tracks = tracks_res.await;
                let tracks = match tracks {
                    Ok(tracks) => tracks,
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
