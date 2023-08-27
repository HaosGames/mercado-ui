#![allow(non_snake_case)]
use crate::fetchers::*;
use leptos::{html::Input, *};
use leptos_router::*;
use mercado::api::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <PredictionList/>
    }
}
#[component]
pub fn Login(set_access: WriteSignal<Option<AccessRequest>>) -> impl IntoView {
    let (user, set_user) = create_signal(String::from(""));
    let challenge = create_local_resource(move || user.get(), get_login_challenge);
    let (signature, set_signature) = create_signal(String::from(""));
    let result =
        create_local_resource(move || (user.get(), signature.get(), set_access), try_login);

    let user_input: NodeRef<Input> = create_node_ref();
    let signature_input: NodeRef<Input> = create_node_ref();

    view! {
        <div>
            <label>"User secp256k1 public key "<input
                type="text"
                value=move || user.get()
                node_ref=user_input
            /></label>
            <button on:click=move |_| {
                let value = user_input.get().unwrap().value();
                set_user.set(value);
            }>"Get singing challenge"</button>
            <p>"Sign the following message: "{move || challenge.read().transpose().ok().flatten()}</p>
            <label>"ECDSA Signature: "<input
                type="text"
                value=move || signature.get()
                node_ref=signature_input
            /></label>
            <button on:click=move |_| {
                let value = signature_input.get().unwrap().value();
                set_signature.set(value);
            }>"Login"</button>
            <p>{move || result.read().transpose().ok().flatten() }</p>
        </div>
    }
}
#[component]
pub fn PredictionListItem(prediction: PredictionOverviewResponse) -> impl IntoView {
    view! {
        <tr>
            <td><a href={format!("prediction/{}", prediction.id)}>{prediction.name}</a></td>
            <td>{prediction.trading_end.to_string()}</td>
            <td>{prediction.judge_share_ppm / 10000}"%"</td>
        </tr>
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
                    <p>{predictions.len()}" prediction(s)"</p>
                    <table role="grid">
                        <tr>
                           <th>"Prediction"</th>
                           <th>"End"</th>
                           <th>"Judge Share"</th>
                        </tr>
                        {
                            predictions.sort_by(|a, b| a.id.cmp(&b.id));
                            predictions.into_iter()
                            .map(|prediction| view! {<PredictionListItem prediction=prediction/>})
                            .collect::<Vec<_>>()
                        }
                    </table>
                }.into_view(),
                Some(Err(e)) => view! {<p>{format!("Got error: {:?}", e)}</p>}.into_view(),
            }
        }
        </div>
    }
}

#[component]
pub fn PredictionOverview(access: ReadSignal<Option<AccessRequest>>) -> impl IntoView {
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
                        <h3>{prediction.name}</h3>
                        <p>"State: "{prediction.state.to_string()}<br/>
                        "End: "{prediction.trading_end.to_string()}<br/>
                        "Judge share: "{prediction.judge_share_ppm/10000}"%"<br/>
                        "Decision period: "{prediction.decision_period_sec/86400}" days"<br/>
                        </p>
                        <JudgeList prediction=prediction.id judge_count=prediction.judge_count access=access/>
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
pub fn JudgeList(
    prediction: RowId,
    judge_count: u32,
    access: ReadSignal<Option<AccessRequest>>,
) -> impl IntoView {
    let judges = create_local_resource(
        move || prediction,
        move |prediction| get_prediction_judges(prediction),
    );
    view! {
        {
            move || match judges.read() {
                None => view! {<p>"Loading..."</p>}.into_view(),
                Some(Ok(judges)) => view! {
                    <details open>
                        <summary>{format!("Judges: {}/{}", judge_count, judges.len())}</summary>
                        <table>
                            <tr>
                                <th>"Judge"</th>
                                <th>"State"</th>
                                <th>"Actions"</th>
                            </tr>
                            <For each=move || judges.clone() key=move |judge| judge.user
                            view=move |judge: Judge| view!{
                                <JudgeListItem judge=judge access=access />
                            }/>
                        </table>
                    </details>
                }.into_view(),
                Some(Err(e)) => view! {<p>{format!("Got error: {:?}", e)}</p>}.into_view(),

            }
        }
    }
}
#[component]
pub fn JudgeListItem(judge: Judge, access: ReadSignal<Option<AccessRequest>>) -> impl IntoView {
    let accept = create_action(|request: &PostRequest<AcceptNominationRequest>| {
        accept_nomination(request.data.clone(), request.access)
    });
    let refuse = create_action(|request: &PostRequest<AcceptNominationRequest>| {
        refuse_nomination(request.data.clone(), request.access)
    });
    view! {
        <tr>
            <td>{move || judge.user.to_string()}</td>
            <td>{move || judge.state.to_string()}</td>
            <td>
                <a href="#" role="button" class="outline" on:click=move |_|
                    accept.dispatch(PostRequest {
                        data: AcceptNominationRequest {user: judge.user, prediction: judge.prediction},
                        access: access.get().unwrap()})>
                    "Accept"
                </a>
                <a href="#" role="button" class="outline contrast" on:click=move |_|
                    refuse.dispatch(PostRequest {
                        data: AcceptNominationRequest {user: judge.user, prediction: judge.prediction},
                        access: access.get().unwrap()})>
                    "Refuse"
                </a>
            </td>
        </tr>
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
                    <details>
                        <summary>{format!("Bets: {}", bets.len())}</summary>
                        <table>
                            <tr>
                                <th>"Bet"</th>
                                <th>"Amount"</th>
                                <th>"User"</th>
                            </tr>
                            <For each=move || bets.clone() key=move |judge| judge.user
                            view=move |bet: Bet| view!{
                                <tr>
                                    <td>{bet.bet}</td>
                                    <td>{bet.amount.unwrap_or(0)}</td>
                                    <td>{bet.user.to_string()}</td>
                                </tr>
                            }/>
                        </table>
                    </details>
                }.into_view(),
                Some(Err(e)) => view! {<p>{format!("Got error: {:?}", e)}</p>}.into_view(),

            }
        }
    }
}
#[component]
pub fn Username(user: Option<UserPubKey>) -> impl IntoView {
    let mut names = vec![];
    if let Some(user) = user {
        names.push(user);
    }
    let usernames = create_local_resource(move || names.clone(), get_usernames);

    view! {
        {
            move || {
                if let Some(user) = user {
                    let name = usernames.read().transpose().ok().flatten().unwrap_or_default()
                        .get(&user).cloned().unwrap_or(user.to_string());
                    if name.is_empty() {
                        user.to_string()
                    } else {name}
                } else {
                    "User".to_string()
                }
            }
        }
        ""
    }
}
