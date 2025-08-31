use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct SearchPayload {
    query: String,
    top_k: Option<usize>,
}

#[derive(Deserialize)]
struct SearchResult {
    id: String,
    score: f32,
    metadata: serde_json::Value,
}

#[component]
pub fn Search(cx: Scope) -> impl IntoView {
    let query = create_rw_signal(cx, String::new());
    let results = create_rw_signal(cx, vec![] as Vec<SearchResult>);
    let status = create_rw_signal(cx, String::new());

    let on_search = move |ev: web_sys::Event| {
        ev.prevent_default();
        let q = query.get().eval().to_string();
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

    view! { cx,
        <form on:submit=on_search>
            <input prop:value=query.get() on:input=move |e| query.set(event_target_value(&e)) />
            <button type="submit">"Search"</button>
        </form>
        <p>{ move || status.get() }</p>
        <ul>
            { move || results.get().iter().map(|r| view!{ cx, <li>{format!("{} (score {})", r.id, r.score)}</li> }).collect::<Vec<_>>() }
        </ul>
    }
}
