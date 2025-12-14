use leptos::{logging::log, mount, prelude::*,  task::spawn_local};
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

use crate::frontend::{calendar::Calendar, jamselector::JamSelector, track_list::TrackList, webrtc::WebRtcComp};



pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/repan_stream.css" />

        // sets the document title
        <Title text="Welcome to Leptos" />

        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage />
                </Routes>
            </main>
        </Router>
    }
}


/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {

    let (started, set_started) = signal(false);

    let (selected_day, set_selected_day) = signal(String::new());
    let (selected_jam_id, set_selected_jam_id) = signal(0 as i64);



    provide_context(set_selected_day);

    view! {
        <Router>
            <h1>"Welcome to Repan!"</h1>
            <nav>
                <a href="/">"Home"</a>
            </nav>
        </Router>
        <Calendar></Calendar>
        <JamSelector
            selected_day=selected_day
            set_selected_jam_id=set_selected_jam_id
        ></JamSelector>
        <TrackList selected_jam_id=selected_jam_id />
        <Show when=move || { !started.get() } fallback=|| view! { <p>"Connect"</p> }>
            <button on:click=move |_| {
                spawn_local(async {
                    //start_connecting().await;
                });
                *set_started.write() = true;
            }>"Start Connecting"</button>
        </Show>
        <button on:click=move |_| {
            spawn_local(async {});
        }>"Play pipeline"</button>
        <WebRtcComp></WebRtcComp>
    }

}
