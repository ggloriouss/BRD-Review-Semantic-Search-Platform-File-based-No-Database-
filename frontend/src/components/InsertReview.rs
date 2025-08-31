use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct InsertPayload {
    title: Option<String>,
    body: String,
    rating: Option<u8>,
}

#[component]
pub fn InsertReview(cx: Scope) -> impl IntoView {
    let title = create_rw_signal(cx, String::new());
    let body = create_rw_signal(cx, String::new());
    let rating = create_rw_signal(cx, None::<u8>);
    let status = create_rw_signal(cx, String::new());

    let on_submit = move |ev: web_sys::Event| {
        ev.prevent_default();
        let t = title.get().eval().to_string();
        let b = body.get().eval().to_string();
        let r = rating.get().get();
        let payload = InsertPayload { title: if t.is_empty() { None } else { Some(t) }, body: b.clone(), rating: r };

        // call backend (example using wasm fetch simplified)
        spawn_local(async move {
            // replace with proper fetch helper
            let res: Result<serde_json::Value, String> = crate::api::post_json("/reviews", &payload).await;
            match res {
                Ok(v) => status.set(format!("OK: {:?}", v)),
                Err(e) => status.set(format!("ERR: {}", e)),
            }
        });
    };

    view! { cx,
        <form on:submit=on_submit>
            <input prop:value=title.get() on:input=move |e| title.set(event_target_value(&e)) />
            <textarea prop:value=body.get() on:input=move |e| body.set(event_target_value(&e)) />
            <button type="submit">"Upload"</button>
        </form>
        <p>{ move || status.get() }</p>
    }
}