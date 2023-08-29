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
pub fn Navi(
    access: ReadSignal<Option<AccessRequest>>,
    set_access: WriteSignal<Option<AccessRequest>>,
) -> impl IntoView {
    let check_login = create_local_resource(move || access.get(), check_login);
    view! {
        <nav class="container">
            <ul>
                <details role="list" >
                    <summary aria-haspopup="listbox" role="link" >"New"</summary>
                    <ul role="listbox">
                        <li><a>"Prediction"</a></li>
                        <li><a>"Bet"</a></li>
                    </ul>
                </details>
            </ul>
            <ul>
            <li><a href="/"><strong>"Mercado"</strong></a></li>
            </ul>
            <ul><li>{
                move || if access.get().is_some() && check_login.read().transpose().ok().flatten().is_some() {
                    view!{
                        <details role="list" >
                            <summary aria-haspopup="listbox" role="link" ><Username user={
                                if let Some(access) = access.get() {
                                    Some(access.user)
                                } else {
                                    None
                                }
                            } /></summary>
                            <ul role="listbox">
                                <li><a>"Edit user"</a></li>
                                <li><a>"Predictions"</a></li>
                                <li><a href="/my_bets">"Bets"</a></li>
                                <li><a>"Judges"</a></li>
                                <li><a href="/" on:click=move |_| {set_access.set(None)} >"Logout"</a></li>
                            </ul>
                        </details>
                    }.into_view()
                } else {
                    view!{
                        <a href="/login">"Login"</a>
                    }.into_view()
                }
            }</li></ul>
        </nav>
    }
}
#[component]
pub fn Login(set_access: WriteSignal<Option<AccessRequest>>) -> impl IntoView {
    let (user, set_user) = create_signal(String::from(""));
    let challenge = create_local_resource(move || user.get(), create_login_challenge);
    let (signature, set_signature) = create_signal(String::from(""));
    let result = create_local_resource(
        move || {
            (
                user.get(),
                signature.get(),
                challenge
                    .read()
                    .transpose()
                    .ok()
                    .flatten()
                    .unwrap_or_default(),
                set_access,
            )
        },
        try_login,
    );

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
        <UnwrapResource t=move || predictions.read() view=move |mut predictions| view! {
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
        } />
    }
}
#[component]
pub fn UnwrapResource<F, V, T, W>(view: F, t: W) -> impl IntoView
where
    F: Fn(T) -> V + 'static,
    W: Fn() -> Option<Result<T, String>> + 'static,
    V: IntoView,
{
    view! {
        {
            move || match t() {
                None => view! {<p>"Loading..."</p>}.into_view(),
                Some(Ok(t)) => view(t).into_view(),
                Some(Err(e)) => view! {<p>{format!("{:?}", e)}</p>}.into_view(),
            }
        }
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
        <UnwrapResource t=move || prediction.read() view=move |prediction| view! {
            <h3>{prediction.name}</h3>
            <p>"State: "{prediction.state.to_string()}<br/>
            "End: "{prediction.trading_end.to_string()}<br/>
            "Judge share: "{prediction.judge_share_ppm/10000}"%"<br/>
            "Decision period: "{prediction.decision_period_sec/86400}" days"<br/>
            </p>
            <JudgeList prediction=prediction.id judge_count=prediction.judge_count access=access/>
            <BetList prediction=prediction.id user=None />
            <p>"Id: "{prediction.id}</p>
        } />
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
        move |prediction| get_judges(Some(prediction), None),
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
                            view=move |judge: JudgePublic| view!{
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
pub fn JudgeListItem(
    judge: JudgePublic,
    access: ReadSignal<Option<AccessRequest>>,
) -> impl IntoView {
    let accept = create_action(|request: &PostRequest<NominationRequest>| {
        accept_nomination(request.data.clone(), request.access.clone())
    });
    let refuse = create_action(|request: &PostRequest<NominationRequest>| {
        refuse_nomination(request.data.clone(), request.access.clone())
    });
    let (count, set_count) = create_signal(0);
    let judge_priv = create_local_resource(
        move || count.get(),
        move |_| get_judge(judge.prediction, judge.user, access),
    );
    view! {
        <tr>
            <td>{move || judge.user.to_string()}</td>
            <td><UnwrapResource t=move || judge_priv.read() view=move |judge| judge.state.to_string() /></td>
            <td><UnwrapResource t=move || judge_priv.read() view= move |judge| view! {
                <a href="#" role="button" class="outline" on:click=move |_| {
                    accept.dispatch(PostRequest {
                        data: NominationRequest {user: judge.user, prediction: judge.prediction},
                        access: access.get().unwrap()});
                    set_count.set(count.get() + 1);
                }>
                    "Accept"
                </a>
                <a href="#" role="button" class="outline contrast" on:click=move |_| {
                    refuse.dispatch(PostRequest {
                        data: NominationRequest {user: judge.user, prediction: judge.prediction},
                        access: access.get().unwrap()});
                    set_count.set(count.get() + 1);
                }>
                    "Refuse"
                </a>
            } />
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
    let usernames = create_local_resource(move || user.unwrap(), get_username);

    view! {{
        move || {
            if let Some(user) = user {
                let name = usernames.read().transpose().ok().flatten().unwrap_or_default();
                if name.is_empty() {
                    user.to_string()
                } else {name}
            } else {
                "User".to_string()
            }
        }.into_view()
    }}
}
#[component]
pub fn MyBets(access: ReadSignal<Option<AccessRequest>>) -> impl IntoView {
    let bets = create_local_resource(move || access.get(), my_bets);

    view! {
        <table>
            <tr>
                <th>"Bet"</th>
                <th>"Amount"</th>
                <th>"Prediction"</th>
                <th>"State"</th>
                <th>"Actions"</th>
            </tr>
            <For each=move || bets.read().transpose().ok().flatten().unwrap_or_default() key=move |bet| bet.user
            view=move |bet: Bet| view!{
                <tr>
                    <td>{bet.bet}</td>
                    <td>{bet.amount.unwrap_or(0)}</td>
                    <td>"Prediction"</td>
                    <td>{bet.state.to_string()}</td>
                    <td>""</td>
                </tr>
            }/>
        </table>
    }
}
