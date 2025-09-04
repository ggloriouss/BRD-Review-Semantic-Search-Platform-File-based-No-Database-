use leptos::*;
// use leptos_router::*;

mod api;
mod components;

use components::{insert_review::InsertReview, search::Search};

#[component]
fn App() -> impl IntoView {
    view! {
        <div>
            <h1>"Review Semantic Search (File-based)"</h1>
            <InsertReview />
            <hr/>
            <Search />
        </div>
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // SSR or native: do nothing
}