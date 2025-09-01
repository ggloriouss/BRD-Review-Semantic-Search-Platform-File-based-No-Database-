use leptos::*;
use leptos::ev::SubmitEvent;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct InsertPayload {
    title: Option<String>,
    body: String,
    rating: Option<u8>,
}

#[component]
pub fn InsertReview() -> impl IntoView {
    let title = create_rw_signal(String::new());
    let body = create_rw_signal(String::new());
    let rating = create_rw_signal(None::<u8>);
    let status = create_rw_signal(String::new());

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let t = title.get();
        let b = body.get();
        let r = rating.get();
        let payload = InsertPayload { 
            title: if t.is_empty() { None } else { Some(t) }, 
            body: b.clone(), 
            rating: r 
        };

        spawn_local(async move {
            let res: Result<serde_json::Value, String> = crate::api::post_json("/reviews", &payload).await;
            match res {
                Ok(v) => status.set(format!("OK: {:?}", v)),
                Err(e) => status.set(format!("ERR: {}", e)),
            }
        });
    };

    view! {
        <h2>"Upload/Insert Review"</h2>
        <form on:submit=on_submit>
            <input 
                placeholder="Title (optional)" 
                prop:value=title 
                on:input=move |e| title.set(event_target_value(&e)) 
            />
            <textarea 
                placeholder="Body" 
                prop:value=body 
                on:input=move |e| body.set(event_target_value(&e)) 
            />
            <input 
                type="number" 
                placeholder="Rating (1-5)" 
                on:input=move |e| rating.set(Some(event_target_value(&e).parse().unwrap_or(0))) 
            />
            <button type="submit">"Upload"</button>
        </form>
        <p>{move || status.get()}</p>
    }
}