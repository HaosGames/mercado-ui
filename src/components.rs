#![allow(non_snake_case, unused)]
use crate::fetchers::*;
use leptos::*;
use leptos_router::*;
use mercado::api::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <PredictionList/>
    }
}
#[component]
pub fn PredictionListItem(prediction: PredictionOverviewResponse) -> impl IntoView {
    view! {
        <li>
            <a href={format!("prediction/{}", prediction.id)}>{prediction.name}</a>
            <p>"Ends "{prediction.trading_end.to_string()}
            " | Judge share: "{prediction.judge_share_ppm / 10000}"%"</p>
        </li>
    }
}
#[component]
pub fn PredictionList() -> impl IntoView {
    let predictions = create_local_resource(move || {}, get_predictions);

    view! {
        <div>
        {
            move || match predictions.read() {
                None => view! {<p>"Loading..."</p>}.into_view(),
                Some(Ok(mut predictions)) => view! {
                    <div>
                        <p>{predictions.len()}" prediction(s)"</p>
                        <ul>{
                            predictions.sort_by(|a, b| a.id.cmp(&b.id));
                            predictions.into_iter()
                            .map(|prediction| view! {<PredictionListItem prediction=prediction/>})
                            .collect::<Vec<_>>()
                        }</ul>
                    </div>
                }.into_view(),
                Some(Err(e)) => view! {<p>{format!("Got error: {:?}", e)}</p>}.into_view(),
            }
        }
        </div>
    }
}

#[component]
pub fn PredictionOverview() -> impl IntoView {
    let params = use_params_map();
    let prediction = create_local_resource(
        move || params.with(|p| p.get("id").cloned().unwrap_or_default()),
        move |id| get_prediction_overview(id.parse().unwrap_or_default()),
    );
    let judges = create_local_resource(
        move || params.with(|p| p.get("id").cloned().unwrap_or_default()),
        move |id| get_prediction_judges(id.parse().unwrap_or_default()),
    );
    view! {
        <div>
        {
            move || match prediction.read() {
                None => view! {<p>"Loading..."</p>}.into_view(),
                Some(Ok(prediction)) => view! {
                    <div>
                        <h2>{prediction.name}</h2>
                        <p>{format!("End: {} | Judge share: {}% | Decision period: {} days",
                                    prediction.trading_end,
                                    prediction.judge_share_ppm/10000,
                                    prediction.decision_period_sec / 86400
                            )}</p>
                        <p>{format!("Judges: {}/{}", prediction.judge_count, 0)}</p>
                    </div>
                }.into_view(),
                Some(Err(e)) => view! {<p>{format!("Got error: {:?}", e)}</p>}.into_view(),
            }
        }
        </div>

    }
}
#[component]
pub fn BetListItem(bet: Bet) -> impl IntoView {
    view! {
        <li>
        {format!("{}: {} sats", bet.bet, bet.amount.unwrap_or(0))}
        </li>
    }
}
#[component]
pub fn JudgeList(judges: Vec<Judge>) -> impl IntoView {}
