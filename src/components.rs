#![allow(non_snake_case, unused)]
use crate::fetchers::*;
use leptos::*;
use leptos_router::*;
use mercado::api::*;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    view! {cx,
        <PredictionList/>
    }
}
#[component]
pub fn PredictionListItem(cx: Scope, prediction: PredictionOverviewResponse) -> impl IntoView {
    view! {cx,
        <li>
            <a href={format!("prediction/{}", prediction.id)}>{prediction.name}</a>
            <p>"Ends "{prediction.trading_end.to_string()}
            " | Judge share: "{prediction.judge_share_ppm / 10000}"%"</p>
        </li>
    }
}
#[component]
pub fn PredictionList(cx: Scope) -> impl IntoView {
    let predictions = create_local_resource(cx, move || {}, get_predictions);

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
        </div>
    }
}

#[component]
pub fn PredictionOverview(cx: Scope) -> impl IntoView {
    let params = use_params_map(cx);
    let prediction = create_local_resource(
        cx,
        move || params.with(|p| p.get("id").cloned().unwrap_or_default()),
        move |id| get_prediction_overview(id.parse().unwrap_or_default()),
    );
    let judges = create_local_resource(
        cx,
        move || params.with(|p| p.get("id").cloned().unwrap_or_default()),
        move |id| get_prediction_judges(id.parse().unwrap_or_default()),
    );
    view! {cx,
        <div>
        {
            move || match prediction.read(cx) {
                None => view! {cx, <p>"Loading..."</p>}.into_view(cx),
                Some(Ok(prediction)) => view! {cx,
                    <div>
                        <h2>{prediction.name}</h2>
                        <p>{format!("End: {} | Judge share: {}% | Decision period: {} days",
                                    prediction.trading_end,
                                    prediction.judge_share_ppm/1000,
                                    prediction.decision_period_sec / 86400
                            )}</p>
                        <p>{format!("Judges: {}/{}", prediction.judge_count, 0)}</p>
                    </div>
                }.into_view(cx),
                Some(Err(e)) => view! {cx, <p>{format!("Got error: {:?}", e)}</p>}.into_view(cx),
            }
        }
        </div>

    }
}
#[component]
pub fn BetListItem(cx: Scope, bet: Bet) -> impl IntoView {
    view! {cx,
        <li>
        {format!("{}: {} sats", bet.bet, bet.amount.unwrap_or(0))}
        </li>
    }
}
#[component]
pub fn JudgeList(cx: Scope, judges: Vec<Judge>) -> impl IntoView {}
