use leptos::*;
use leptos_router::*;

mod api;
mod components;

#[component]
pub fn App() -> impl IntoView { // Remove cx: Scope
    view! { // Remove cx from view!
        <Router>
            <nav>
                <A href="/">"Home"</A>
                <A href="/insert">"Index (Upload)"</A>
                <A href="/search">"Search"</A>
            </nav>
            <main>
                <Routes>
                    <Route path="/" view=move || view! { <h1>"Welcome to Review Search"</h1> } />
                    <Route path="/insert" view=move || view! { <components::insert_review::InsertReview /> } />
                    <Route path="/search" view=move || view! { <components::search::Search /> } />
                </Routes>
            </main>
        </Router>
    }
}

fn main() {
    mount_to_body(|| view! { <App /> });
}