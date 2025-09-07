use leptos::*;
use crate::api::{
    create_bulk, create_review, get_paths, set_paths, Paths, ReviewInput,
};

#[component]
pub fn InsertReview() -> impl IntoView {
    // review form
    let review = create_rw_signal(String::new());
    let rating = create_rw_signal(String::new());
    let category = create_rw_signal(String::new());

    // bulk
    let bulk_text = create_rw_signal(String::new());

    // paths UI
    let index_path = create_rw_signal(String::new());
    let jsonl_path = create_rw_signal(String::new());
    let map_path = create_rw_signal(String::new());

    let message = create_rw_signal(String::new());

    // Load current paths on mount
    create_effect(move |_| {
        spawn_local(async move {
            match get_paths().await {
                Ok(p) => {
                    index_path.set(p.index_path);
                    jsonl_path.set(p.jsonl_path);
                    map_path.set(p.map_path);
                }
                Err(e) => message.set(format!("Failed to load paths: {:?}", e)),
            }
        });
    });

    let on_save_paths = move |_| {
        let p = Paths {
            index_path: index_path.get(),
            jsonl_path: jsonl_path.get(),
            map_path: map_path.get(),
        };
        let message = message.clone();
        spawn_local(async move {
            match set_paths(&p).await {
                Ok(_) => message.set("Paths updated successfully.".into()),
                Err(e) => message.set(format!("Update paths error: {:?}", e)),
            }
        });
    };

    let on_submit = move |_| {
        let review_v = review.get();
        let rating_v = rating.get();
        let category_v = category.get();
        let message = message.clone();
        spawn_local(async move {
            let rating_parsed = rating_v.trim().parse::<i32>().unwrap_or(0);
            let category_opt = {
                let t = category_v.trim();
                if t.is_empty() { None } else { Some(t.to_string()) }
            };
            let input = ReviewInput {
                review: review_v,
                rating: rating_parsed,
                category: category_opt,
            };
            match create_review(&input).await {
                Ok(r) => message.set(format!("Inserted review id={} vector_id={}", r.id, r.vector_id)),
                Err(e) => message.set(format!("Error: {:?}", e)),
            }
        });
    };

    let on_bulk = move |_| {
        let raw = bulk_text.get();
        let message = message.clone();
        spawn_local(async move {
            let parsed: Result<Vec<ReviewInput>, _> = serde_json::from_str(&raw);
            match parsed {
                Ok(list) => match create_bulk(&list).await {
                    Ok(res) => message.set(format!("Inserted {} reviews", res.len())),
                    Err(e) => message.set(format!("Bulk error: {:?}", e)),
                },
                Err(e) => message.set(format!("Parse error: {}", e)),
            }
        });
    };

    view! {
        <div class="insert-review">
            <h2>"Storage Paths"</h2>
            <div>
                <label>"Index file:"</label>
                <input prop:value=index_path on:input=move |e| index_path.set(event_target_value(&e)) />
            </div>
            <div>
                <label>"Metadata JSONL:"</label>
                <input prop:value=jsonl_path on:input=move |e| jsonl_path.set(event_target_value(&e)) />
            </div>
            <div>
                <label>"Vector map JSONL:"</label>
                <input prop:value=map_path on:input=move |e| map_path.set(event_target_value(&e)) />
            </div>
            <button on:click=on_save_paths>"Save Paths"</button>

            <h2 style="margin-top:1rem">"Insert Review"</h2>
            <div>
                <label>"Review:"</label>
                <textarea prop:value=review on:input=move |e| review.set(event_target_value(&e)) />
            </div>
            <div>
                <label>"Rating (0-5):"</label>
                <input prop:value=rating on:input=move |e| rating.set(event_target_value(&e)) />
            </div>
            <div>
                <label>"Category (optional):"</label>
                <input
                    placeholder="e.g., Food, Service, Ambience"
                    prop:value=category
                    on:input=move |e| category.set(event_target_value(&e))
                />
            </div>
            <button on:click=on_submit>"Submit"</button>

            <h3 style="margin-top:1rem">"Bulk Insert (JSON Array)"</h3>
            <p class="text-sm opacity-70">
                r#"Example: [{"review":"Nice","rating":5,"category":"Food"},{"review":"Okay","rating":3}]"#
            </p>
            <textarea rows=8 prop:value=bulk_text on:input=move |e| bulk_text.set(event_target_value(&e)) />
            <button on:click=on_bulk>"Bulk Upload"</button>

            <p>{ move || message.get() }</p>
        </div>
    }
}
