use leptos::*;
use leptos_router::*;

mod api;
mod components;

use components::{insert_review::InsertReview, search::Search};

#[component]
fn Home() -> impl IntoView {
    view! {
        <div>
            <h1 class="text-center">"Review Semantic Search (File-based)"</h1>
            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 20px; padding: 20px;">
                <div><InsertReview /></div>
                <div><Search /></div>
            </div>
        </div>
    }
}

#[component]
fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes>
                // Root route: IMPORTANT — in leptos_router the root is "" (not "/")
                <Route path="" view=Home />

                // Optional direct routes (handy if you link to them)
                <Route path="/search" view=Search />
                <Route path="/insert" view=InsertReview />

                // 404 fallback (optional)
                <Route path="/*any" view=|| view! { <p>"Not Found"</p> } />
            </Routes>
        </Router>
    }
}

#[cfg(target_arch = "wasm32")]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn main() {
    // SSR/native path not used in this setup
}
