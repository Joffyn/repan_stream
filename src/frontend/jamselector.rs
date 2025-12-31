use crate::backend::{
    database::JamQueryResult,
    serverfunctions::{get_all_jams_from_day, get_jam_path},
};
use leptos::{logging::log, prelude::*};
use web_sys::RtcDataChannel;

#[component]
pub fn JamSelector(
    selected_day: ReadSignal<String>,
    set_selected_jam_id: WriteSignal<i64>,
) -> impl IntoView {
    let jams_res = Resource::new(
        move || selected_day.get(),
        move |_| async move { get_all_jams_from_day(selected_day.get()).await },
    );
    //let (path, set_path) = signal(String::new());

    //let jam_path = Resource::new(
    //    move || path.get(),
    //    move |_| async move { get_jam_path(path.get()).await },
    //);
    let dc = use_context::<ReadSignal<Option<RtcDataChannel>, LocalStorage>>()
        .expect("to have found getter");
    view! {
        <Suspense fallback=move || {
            view! { <p>"Loading..."</p> }
        }>
            {move || move || Suspend::new(async move {
                let jams = jams_res.await;
                let jams = match jams {
                    Ok(jams) => jams,
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
                    <div class="jam-select">
                        <For each=move || jams.clone() key=|state| state.id.clone() let(child)>
                            <div on:click=move |_| {

                                log!("Pressed");
                                *set_selected_jam_id.write() = child.id.clone();
                                //*set_path.write() = child.data.clone();
                                //let _ = dc.get().unwrap().send_with_str(jam_path.get().unwrap().unwrap().data.as_str()).unwrap();
                            }>{child.data.clone()}</div>
                        </For>
                    </div>
                }
            })}
        </Suspense>
    }
}
