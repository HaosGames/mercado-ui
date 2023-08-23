#![allow(non_snake_case)]
use anyhow::Result;
use leptos::*;
use leptos_router::*;
use mercado::api::*;
use mercado::client::Client;
use mercado::secp256k1::{generate_keypair, rand};

pub fn App(cx: Scope) -> impl IntoView {
    let (count, set_count) = create_signal(cx, 10);
    let predictions = create_local_resource(cx, move || count.get(), get_predictions);

    view! { cx,
        <div>
            <Outlet/>
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
            <a href={format!("prediction/{}", prediction.id)}>{prediction.name}</a>
            <p>"True: "{prediction.bets_true}"sats, "
            "False: "{prediction.bets_false}"sats"
            " | Ends "{prediction.trading_end.to_rfc2822()}
            " | Judge share: "{prediction.judge_share_ppm / 1000}"%"</p>
        </li>
    }
}
async fn get_predictions(_how_many: u32) -> Result<Vec<PredictionListItemResponse>, String> {
    let client = Client::new("http://127.0.0.1:8081".to_string());
    client.get_predictions().await.map_err(map_any_err)
}
async fn get_user_prediction(
    prediction: RowId,
    user: UserPubKey,
) -> Result<UserPredictionOverviewResponse, String> {
    let client = Client::new("http://127.0.0.1:8081".to_string());
    client
        .get_user_prediction(prediction, user)
        .await
        .map_err(map_any_err)
}

#[component]
pub fn UserPredictionOverview(cx: Scope) -> impl IntoView {
    let params = use_params_map(cx);
    let prediction = create_local_resource(
        cx,
        move || params.with(|p| p.get("id").cloned().unwrap_or_default()),
        move |id| {
            get_user_prediction(
                id.parse().unwrap_or_default(),
                generate_keypair(&mut rand::thread_rng()).1,
            )
        },
    );
    view! {cx,
        <div>
        {
            move || match prediction.read(cx) {
                None => view! {cx, <p>"Loading..."</p>}.into_view(cx),
                Some(Ok(prediction)) => view! {cx,
                    <div>
                        <h2>{prediction.name}</h2>
                    </div>
                }.into_view(cx),
                Some(Err(e)) => view! {cx, <p>{format!("Got error: {:?}", e)}</p>}.into_view(cx),
            }
        }
        </div>

    }
}
