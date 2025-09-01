use leptos::*;
use leptos::ev::SubmitEvent;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct SearchPayload {
    query: String,
    top_k: Option<usize>,
}

#[derive(Deserialize, Clone)]
struct SearchResult {
    id: String,
    score: f32,
    metadata: serde_json::Value,
}

#[component]
pub fn Search() -> impl IntoView {
    let query = create_rw_signal(String::new());
    let results = create_rw_signal(vec![] as Vec<SearchResult>);
    let status = create_rw_signal(String::new());

    let on_search = move |ev: SubmitEvent| {
        ev.prevent_default();
        let q = query.get();
        let payload = SearchPayload { query: q.clone(), top_k: Some(10) };
        spawn_local(async move {
            let res: Result<Vec<SearchResult>, String> = crate::api::post_json("/search", &payload).await;
            match res {
                Ok(v) => {
                    status.set("OK".to_string());
                    results.set(v);
                }
                Err(e) => status.set(format!("ERR: {}", e)),
            }
        });
    };

    view! {
        <h2>"Semantic Search"</h2>
        <form on:submit=on_search>
            <input placeholder="Enter query" prop:value=query.get() on:input=move |e| query.set(event_target_value(&e)) />
            <button type="submit">"Search"</button>
        </form>
        <p>{ move || status.get() }</p>
        <ul>
            { move || results.get().iter().map(|r| view! { <li>{format!("ID: {} (Score: {}) - {}", r.id, r.score, r.metadata)}</li> }).collect::<Vec<_>>() }
        </ul>
    }
}
