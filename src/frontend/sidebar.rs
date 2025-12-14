use leptos::prelude::*;

use crate::backend::{database::JamQueryResult, serverfunctions::{get_all_jams, get_tracks}};

#[component]
pub fn Sidebar() -> impl IntoView
{
    let jams_res = Resource::new(|| () , |_| 
        async move { get_all_jams().await});
    let (jam_select, jam_select_set) = signal(0);
    let tracks_res = Resource::new(move || jam_select.get(), |id: i64| get_tracks(id));
    view! {
        <Suspense fallback=move || {
            view! { <p>"Loading jam..."</p> }
        }>
            {move || Suspend::new(async move {
                let jams = jams_res.await;
                let res = match jams {
                    Ok(jams) => jams,
                    Err(_) => {
                        vec![
                            JamQueryResult {
                                id: -1,
                                data: "Failure".to_string(),
                            },
                        ]
                    }
                };

                view! {
                    <For each=move || res.clone() key=|state| state.id.clone() let(child)>
                        <div
                            style="cursor: pointer;"
                            on:click=move |_| jam_select_set(child.id.clone())
                        >
                            {child.data.clone()}
                        </div>
                    </For>
                }
            })}
        </Suspense>
        <Suspense fallback=move || {
            view! { <p>"Loading tracks..."</p> }
        }>
            {move || Suspend::new(async move {
                let tracks_fut = tracks_res.await;
                let loaded_tracks = match tracks_fut {
                    Ok(tracks) => tracks,
                    Err(_) => {
                        vec![
                            JamQueryResult {
                                id: -1,
                                data: "Failure".to_string(),
                            },
                        ]
                    }
                };

                view! {
                    <For each=move || loaded_tracks.clone() key=|state| state.id.clone() let(child)>
                        <div>{child.data.clone()}</div>

                    </For>
                }
            })}
        </Suspense>
    }
}
