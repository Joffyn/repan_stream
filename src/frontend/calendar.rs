use leptos::prelude::*;
use chrono::{Datelike, Month, NaiveDate};
use num_traits::cast::FromPrimitive;

use crate::backend::serverfunctions::get_all_days_with_jams;


#[component]
pub fn Calendar() -> impl IntoView {

    let (month, set_month) = signal(chrono::Local::now().month());
    let (year, set_year) = signal(chrono::Local::now().year());
    // Helper: weekday offset
    let first_weekday_offset = move || {
        let first_day = NaiveDate::from_ymd_opt(year(), month() as u32, 1).unwrap();
        first_day.weekday().num_days_from_sunday()
    };

    // Navigation
    let prev_month = move |_| {
        if month() == 1 {
            set_month(12);
            set_year.update(|y| *y -= 1);
        } else {
            set_month.update(|m| *m -= 1);
        }
    };
    let next_month = move |_| {
        if month() == 12 {
            set_month(1);
            set_year.update(|y| *y += 1);
        } else {
            set_month.update(|m| *m += 1);
        }
    };

    view! {
        <div class="calendar">
            <div class="calendar-header">
                <button on:click=prev_month>{"<"}</button>
                <span>
                    {move || format!("{} {}", Month::from_u32(month()).unwrap().name(), year())}
                </span>
                <button on:click=next_month>{">"}</button>
            </div>

            <div class="calendar-grid">
                // Weekday headers
                {["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]
                    .into_iter()
                    .map(|d| view! { <div class="weekday">{d}</div> })
                    .collect::<Vec<_>>()}
                // Empty slots before first day
                {(0..first_weekday_offset())
                    .map(|_| view! { <div class="empty"></div> })
                    .collect::<Vec<_>>()}
                // Days of month
                {
                    view! { <Days month=month year=year /> }
                }
            </div>
        </div>
    }
}

#[component]
fn Days(month: ReadSignal<u32>, year: ReadSignal<i32>) -> impl IntoView
{
    let set_day = use_context::<WriteSignal<String>>().expect("to have found setter");
    // Helper: days in month
    let days_in_month = move || {
        let first_day = NaiveDate::from_ymd_opt(year(), month() as u32, 1).unwrap();
        let next_month = if month() == 12 {
            NaiveDate::from_ymd_opt(year() + 1, 1, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(year(), month() as u32 + 1, 1).unwrap()
        };
        (next_month - chrono::Duration::days(1)).day()
    };

    let date = move || NaiveDate::from_ymd_opt(year.get(), month.get(), chrono::Local::now().day()).unwrap();

    let jams_res = Resource::new(move || month.get() ,  move|_| 
        async move { 

            get_all_days_with_jams(date().format("%y%m").to_string()).await
        });

    let jams_res = Resource::new(move || month.get() ,  move|_| 
        async move { 

            get_all_days_with_jams(date().format("%y%m").to_string()).await
        });




    view! {
        <Suspense fallback=move || {
            view! { <p>"Loading..."</p> }
        }>
            {move || Suspend::new(async move {
                let jams = jams_res.await;
                let (ids, data) = match jams {
                    Ok(jams) => jams.into_iter().map(|r| (r.id, r.data)).unzip(),
                    Err(_) => (vec![-1], vec![0]),
                };
                let days: Vec<u32> = vec![1..days_in_month() + 1]
                    .into_iter()
                    .flat_map(|r| r.collect::<Vec<u32>>())
                    .collect();

                view! {
                    <For each=move || days.clone() key=|state| state.clone() let(child)>
                        <div
                            class="day"
                            on:click=move |_| {
                                let y = year.get();
                                let m = month.get();
                                let d = child;
                                let full_date = format!("{:02}{:02}{:02}", y % 100, m, d);
                                *set_day.write() = full_date;
                            }
                        >
                            {
                            if data.contains(&child) {
                                view! { <div class="dayhasjams">{child}</div> }.into_view()
                            } else {

                                view! { <div class="daynojams">{child}</div> }
                                    .into_view()
                            }}

                        </div>
                    </For>
                }
            })}
        </Suspense>
    }
}
