#![allow(non_snake_case)]
use anyhow::{anyhow, Result};
use leptos::*;
use mercado::api::*;
use mercado::client::Client;

pub fn App(cx: Scope) -> impl IntoView {
    let (count, set_count) = create_signal(cx, 10);
    let predictions = create_local_resource(cx, move || count.get(), query_predictions);

    view! { cx,
        <div>
        {
            move || match predictions.read(cx) {
                None => view! {cx, <p>"Loading..."</p>}.into_view(cx),
                Some(Ok(predictions)) => view! {cx, <p>{predictions.len()}</p>}.into_view(cx),
                Some(Err(e)) => view! {cx, <p>{format!("Got error: {:?}", e)}</p>}.into_view(cx),
            }
        }
            <button
                on:click=move |_| {
                    set_count.update(|n| *n += 1);
                }
            >
                "Refresh"
            </button>
        </div>
    }
}
async fn query_predictions(_how_many: u32) -> Result<Vec<PredictionListItemResponse>, String> {
    let client = Client::new("http://127.0.0.1:8081".to_string());
    let result = client.get_predictions().await;
    match result {
        Ok(predictions) => Ok(predictions),
        Err(e) => Err(format!("{:#}", e)),
    }
}
