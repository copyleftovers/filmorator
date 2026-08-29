use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

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
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/filmorator.css" />
        <Title text="Filmorator" />

        <Router>
            <main class="min-h-screen bg-gray-50">
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage />
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center min-h-screen p-8">
            <h1 class="text-4xl font-bold text-gray-900 mb-4">"Filmorator"</h1>
            <p class="text-xl text-gray-600 mb-8 text-center max-w-md">
                "Crowdsource photo rankings from your network."
            </p>
            <div class="bg-white rounded-lg shadow-md p-6 max-w-sm w-full">
                <p class="text-gray-700 text-center">
                    "Show 3 photos. Rank best to worst. Repeat."
                </p>
                <p class="text-gray-500 text-sm text-center mt-4">"Coming soon: comparison UI"</p>
            </div>
        </div>
    }
}
