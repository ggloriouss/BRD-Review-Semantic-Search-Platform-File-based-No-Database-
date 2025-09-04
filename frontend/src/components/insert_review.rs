use leptos::*;
// use leptos::ev::SubmitEvent;
use crate::api::{create_review, create_bulk, ReviewInput};

#[component]
pub fn InsertReview() -> impl IntoView {
    let review = create_rw_signal(String::new());
    let rating = create_rw_signal(String::new());
    let bulk_text = create_rw_signal(String::new());
    let message = create_rw_signal(String::new());

    let on_submit = move |_| {
        let review_v = review.get();
        let rating_v = rating.get();
        let message = message.clone();
        spawn_local(async move {
            let rating_parsed = rating_v.trim().parse::<i32>().unwrap_or(0);
            let input = ReviewInput {
                review: review_v,
                rating: rating_parsed,
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
            // Expect JSON array of objects
            let parsed: Result<Vec<ReviewInput>, _> = serde_json::from_str(&raw);
            match parsed {
                Ok(list) => {
                    match create_bulk(&list).await {
                        Ok(res) => message.set(format!("Inserted {} reviews", res.len())),
                        Err(e) => message.set(format!("Bulk error: {:?}", e)),
                    }
                }
                Err(e) => message.set(format!("Parse error: {}", e)),
            }
        });
    };

    view! {
        <div class="insert-review">
            <h2>"Insert Review"</h2>
            <div>
                <label>"Review:"</label>
                <textarea prop:value=review on:input=move |e| review.set(event_target_value(&e)) />
            </div>
            <div>
                <label>"Rating (0-5):"</label>
                <input prop:value=rating on:input=move |e| rating.set(event_target_value(&e)) />
            </div>
            <button on:click=on_submit>"Submit"</button>

            <h3>"Bulk Insert (JSON Array)"</h3>
            <textarea rows=8 prop:value=bulk_text on:input=move |e| bulk_text.set(event_target_value(&e)) />
            <button on:click=on_bulk>"Bulk Upload"</button>

            <p>{ move || message.get() }</p>
        </div>
    }
}