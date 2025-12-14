use leptos::prelude::*;
use crate::backend::{database::JamQueryResult, serverfunctions::get_all_jams_from_day};

#[component]
pub fn JamSelector(selected_day: ReadSignal<String>, set_selected_jam_id: WriteSignal<i64>) -> impl IntoView 
{

    let jams_res = Resource::new(move || selected_day.get() ,  move|_| 
        async move 
        { 
            get_all_jams_from_day(selected_day.get()).await
        });
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
                                *set_selected_jam_id.write() = child.id.clone();
                            }>{child.data.clone()}</div>
                        </For>
                    </div>
                }
            })}
        </Suspense>
    }
}
