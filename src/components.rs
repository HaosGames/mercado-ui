#![allow(non_snake_case)]
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
    view! {
        <div>
        {
            move || match prediction.read() {
                None => view! {<p>"Loading..."</p>}.into_view(),
                Some(Ok(prediction)) => view! {
                    <div>
                        <h2>{prediction.name}</h2>
                        <p>{format!("State: {}", prediction.state)}</p>
                        <p>{format!("End: {} | Judge share: {}% | Decision period: {} days",
                                    prediction.trading_end,
                                    prediction.judge_share_ppm/10000,
                                    prediction.decision_period_sec / 86400
                            )}</p>
                        <JudgeList prediction=prediction.id judge_count=prediction.judge_count/>
                        <BetList prediction=prediction.id user=None />
                    </div>
                }.into_view(),
                Some(Err(e)) => view! {<p>{format!("Got error: {:?}", e)}</p>}.into_view(),
            }
        }
        </div>

    }
}
#[component]
pub fn JudgeList(prediction: RowId, judge_count: u32) -> impl IntoView {
    let judges = create_local_resource(
        move || prediction,
        move |prediction| get_prediction_judges(prediction),
    );
    view! {
        {
            move || match judges.read() {
                None => view! {<p>"Loading..."</p>}.into_view(),
                Some(Ok(judges)) => view! {
                    <div>
                        <p>{format!("Judges: {}/{}", judge_count, judges.len())}</p>
                        <ul>
                            <For each=move || judges.clone() key=move |judge| judge.user
                            view=move |judge: Judge| view!{
                                <JudgeListItem judge=judge />
                            }/>
                        </ul>
                    </div>
                }.into_view(),
                Some(Err(e)) => view! {<p>{format!("Got error: {:?}", e)}</p>}.into_view(),

            }
        }
    }
}
#[component]
pub fn JudgeListItem(judge: Judge) -> impl IntoView {
    let accept = create_action(|judge: &Judge| {
        accept_nomination(AcceptNominationRequest {
            prediction: judge.prediction,
            user: judge.user,
        })
    });
    view! {
        <li>
            {format!("{} | {} ", judge.user, judge.state)}
            <button type="submit" on:click=move |_| accept.dispatch(judge.clone())>"Accept Nomination"</button>
        </li>
    }
}
#[component]
pub fn BetList(prediction: RowId, user: Option<UserPubKey>) -> impl IntoView {
    let bets = create_local_resource(
        move || PredictionRequest { prediction, user },
        move |request| get_prediction_bets(request),
    );
    view! {
        {
            move || match bets.read() {
                None => view! {<p>"Loading..."</p>}.into_view(),
                Some(Ok(bets)) => view! {
                    <div>
                        <p>{format!("Bets: {}", bets.len())}</p>
                        <ul>
                            <For each=move || bets.clone() key=move |judge| judge.user
                            view=move |bet: Bet| view!{
                                <li>{format!("{} | {} sats | {}", bet.bet, bet.amount.unwrap_or(0), bet.user)}</li>
                            }/>
                        </ul>
                    </div>
                }.into_view(),
                Some(Err(e)) => view! {<p>{format!("Got error: {:?}", e)}</p>}.into_view(),

            }
        }
    }
}
