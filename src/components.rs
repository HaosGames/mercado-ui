#![allow(non_snake_case)]
use anyhow::Result;
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
                Some(Ok(mut predictions)) => view! {cx,
                    <div>
                        <p>{predictions.len()}" prediction(s)"</p>
                        <ul>{
                            predictions.sort_by(|a, b| a.id.cmp(&b.id));
                            predictions.into_iter()
                            .map(|prediction| view! {cx, <PredictionListItem prediction=prediction/>})
                            .collect::<Vec<_>>()
                        }</ul>
                    </div>
                }.into_view(cx),
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
#[component]
pub fn PredictionListItem(cx: Scope, prediction: PredictionListItemResponse) -> impl IntoView {
    view! {cx,
        <li>
            <p>{prediction.name}</p>
            <p>" | True: "{prediction.bets_true}"sats, "
            "False: "{prediction.bets_false}"sats"
            " | Ends "{prediction.trading_end.to_rfc2822()}
            " | Judge share: "{prediction.judge_share_ppm / 1000}"%"</p>
        </li>
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
