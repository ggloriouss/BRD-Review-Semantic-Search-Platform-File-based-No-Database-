use leptos::*;
// use leptos::ev::SubmitEvent;
use crate::api::{search, SearchRequest, SearchHit};

#[component]
pub fn Search() -> impl IntoView {
    let query = create_rw_signal(String::new());
    let results = create_rw_signal::<Vec<SearchHit>>(vec![]);
    let message = create_rw_signal(String::new());

    let on_search = move |_| {
        let q = query.get();
        if q.trim().is_empty() {
            message.set("Query empty".into());
            return;
        }
        let results_sig = results.clone();
        let message_sig = message.clone();
        spawn_local(async move {
            match search(&SearchRequest { query: q, top_k: Some(10) }).await {
                Ok(resp) => {
                    results_sig.set(resp.hits);
                    message_sig.set(String::new());
                }
                Err(e) => message_sig.set(format!("Error: {:?}", e)),
            }
        });
    };

    view! {
        <div class="search">
            <h2>"Semantic Search"</h2>
            <input prop:value=query on:input=move |e| query.set(event_target_value(&e)) placeholder="Enter query..." />
            <button on:click=on_search>"Search"</button>
            <p>{ move || message.get() }</p>
            <ul>
                <For
                    each=move || results.get()
                    key=|hit| hit.review.id.clone()
                    children=move |hit: SearchHit| {
                        view! {
                            <li>
                                <span>{format!("Score: {:.4}", hit.score)}</span>
                                <div>{hit.review.review.clone()}</div>
                                <small>{format!("Rating: {} | vector_id={} | id={}", hit.review.rating, hit.review.vector_id, hit.review.id)}</small>
                            </li>
                        }
                    }
                />
            </ul>
        </div>
    }
}