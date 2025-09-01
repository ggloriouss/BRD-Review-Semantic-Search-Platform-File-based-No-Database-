// Simple fetch wrappers (WASM / browser)
use serde::Serialize;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::JsCast;
use web_sys::window;

pub async fn post_json<T: Serialize, R: for<'de> serde::Deserialize<'de>>(url: &str, body: &T) -> Result<R, String> {
    let window = window().ok_or("no window")?;
    let headers = web_sys::Headers::new().map_err(|_| "headers err")?;
    headers.append("Content-Type", "application/json").map_err(|_| "header append err")?;
    let resp = window.fetch_with_str_and_init(url, &{
        let mut opts = web_sys::RequestInit::new();
        opts.method("POST");
        opts.headers(&headers);
        opts.body(Some(&wasm_bindgen::JsValue::from_str(&serde_json::to_string(body).map_err(|e| e.to_string())?)));
        opts
    }); // Remove .map_err and ?

    let resp = wasm_bindgen_futures::JsFuture::from(resp).await.map_err(|e| format!("fetch err {:?}", e))?;
    let response: web_sys::Response = resp.dyn_into().map_err(|_| "not a response")?;
    let text = wasm_bindgen_futures::JsFuture::from(response.text().map_err(|_| "no text")?).await.map_err(|_| "text err")?;
    let s = text.as_string().ok_or("no string")?;
    serde_json::from_str(&s).map_err(|e| e.to_string())
}