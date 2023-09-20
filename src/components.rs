#![allow(non_snake_case)]
use crate::{fetchers::*, MercadoState};
use anyhow::bail;
use chrono::DateTime;
use chrono::{offset::Utc, Duration};
use leptos::{html::Input, *};
use leptos_router::*;
use mercado::{
    api::*,
    secp256k1::{generate_keypair, rand},
};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <PredictionList/>
    }
}
#[component]
pub fn Navi(
    state: ReadSignal<MercadoState>,
    set_state: WriteSignal<MercadoState>,
) -> impl IntoView {
    let check_login = create_local_resource(move || state.get().access, check_login);
    view! {
        <nav class="container">
            <ul>
                <details role="list" >
                    <summary aria-haspopup="listbox" role="link" >"New"</summary>
                    <ul role="listbox">
                        <li><A href="/new_prediction">"Prediction"</A></li>
                        <li><A href="/add_bet">"Bet"</A></li>
                    </ul>
                </details>
            </ul>
            <ul>
            <li><a href="/"><strong>"Mercado"</strong></a></li>
            </ul>
            <ul><li>{
                move || if state.get().access.is_some() && check_login.get().transpose().ok().flatten().is_some() {
                    view!{
                        <details role="list" >
                            <summary aria-haspopup="listbox" role="link" ><Username user={
                                if let Some(access) = state.get().access {
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
                                <li><a href="/" on:click=move |_| {set_state.set(MercadoState::default())} >"Logout"</a></li>
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
pub fn Login(set_state: WriteSignal<MercadoState>) -> impl IntoView {
    let (user, set_user) = create_signal(String::from(""));
    let challenge = create_local_resource(move || user.get(), create_login_challenge);
    let (signature, set_signature) = create_signal(String::from(""));
    let result = create_local_resource(
        move || {
            (
                user.get(),
                signature.get(),
                challenge
                    .get()
                    .transpose()
                    .ok()
                    .flatten()
                    .unwrap_or_default(),
                set_state,
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
            <p>"Sign the following message: "{move || challenge.get().transpose().ok().flatten()}</p>
            <label>"ECDSA Signature: "<input
                type="text"
                value=move || signature.get()
                node_ref=signature_input
            /></label>
            <button on:click=move |_| {
                let value = signature_input.get().unwrap().value();
                set_signature.set(value);
            }>"Login"</button>
            <p>{move || result.get().transpose().ok().flatten() }</p>
        </div>
    }
}
#[component]
pub fn PredictionListItem(prediction: PredictionOverviewResponse) -> impl IntoView {
    view! {
        <tr>
            <td><a href={format!("/prediction/{}", prediction.id)}>{prediction.name}</a></td>
            <td>{prediction.trading_end.to_string()}</td>
            <td>{prediction.judge_share_ppm / 10000}"%"</td>
        </tr>
    }
}
#[component]
pub fn PredictionList() -> impl IntoView {
    let predictions = create_local_resource(move || {}, get_predictions);

    view! {
        <UnwrapResource resource=move || predictions.get() view=move |mut predictions| view! {
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
pub fn UnwrapResource<F, V, T, R>(view: F, resource: R) -> impl IntoView
where
    F: Fn(T) -> V + 'static,
    R: Fn() -> Option<Result<T, String>> + 'static,
    V: IntoView,
{
    view! {
        {
            move || match resource() {
                None => view! {<p>"Loading..."</p>}.into_view(),
                Some(Ok(t)) => view(t).into_view(),
                Some(Err(e)) => view! {<p>{format!("{:?}", e)}</p>}.into_view(),
            }
        }
    }
}
#[component]
pub fn UnwrapResourceForUser<F, V, T, R>(
    view: F,
    resource: R,
    user: UserPubKey,
    state: ReadSignal<MercadoState>,
) -> impl IntoView
where
    F: Fn(T) -> V + 'static,
    R: Fn() -> Option<Result<T, String>> + 'static,
    V: IntoView,
{
    if let Some(access) = state.get_untracked().access {
        if let Some(storage_user) = state.get_untracked().user {
            if let UserRole::User = storage_user.role {
                if user == access.user {
                    view! {
                        <UnwrapResource resource=resource view=view />
                    }
                    .into_view()
                } else {
                    "".into_view()
                }
            } else {
                view! {
                    <UnwrapResource resource=resource view=view />
                }
                .into_view()
            }
        } else {
            "".into_view()
        }
    } else {
        "".into_view()
    }
}

#[component]
pub fn PredictionOverview(state: ReadSignal<MercadoState>) -> impl IntoView {
    let params = use_params_map();
    let id = move || params.with(|p| p.get("id").cloned());
    let id = move || id().unwrap_or_default().parse::<RowId>().unwrap();
    let prediction = create_local_resource(move || id(), move |id| get_prediction_overview(id));
    view! {
        <UnwrapResource resource=move || prediction.get() view=move |prediction| view! {
            <h3>{prediction.name}</h3>
            <p>"State: "{prediction.state.to_string()}<br/>
            "End: "{prediction.trading_end.to_string()}<br/>
            "Judge share: "{prediction.judge_share_ppm/10000}"%"<br/>
            "Decision period: "{prediction.decision_period_sec/86400}" days"<br/>
            </p>
            <JudgeList prediction=prediction.id judge_count=prediction.judge_count state=state/>
            <BetList prediction=prediction.id user=None />
            <p>"Id: "{prediction.id}</p>
        } />
    }
}
#[component]
pub fn JudgeList(
    prediction: RowId,
    judge_count: u32,
    state: ReadSignal<MercadoState>,
) -> impl IntoView {
    let judges = create_local_resource(
        move || prediction,
        move |prediction| get_judges(Some(prediction), None),
    );
    view! {
        <UnwrapResource
            resource=move || judges.get()
            view=move |judges| view! {
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
                        <JudgeListItem judge=judge state=state />
                    }/>
                </table>
            </details>
        } />
    }
}
#[component]
pub fn JudgeListItem(judge: JudgePublic, state: ReadSignal<MercadoState>) -> impl IntoView {
    let accept = create_action(|request: &PostRequest<NominationRequest>| {
        accept_nomination(request.data.clone(), request.access.clone())
    });
    let refuse = create_action(|request: &PostRequest<NominationRequest>| {
        refuse_nomination(request.data.clone(), request.access.clone())
    });
    let (count, set_count) = create_signal(0);
    let judge_priv = create_local_resource(
        move || count.get(),
        move |_| get_judge(judge.prediction, judge.user, state),
    );
    view! {
        <tr>
            <td><Username user= Some(judge.user) /></td>
            <td><UnwrapResourceForUser
                user=judge.user
                state=state
                resource=move || judge_priv.get()
                view=move |judge| judge.state.to_string()
            /></td>
            <td><UnwrapResourceForUser
                user=judge.user
                state=state
                resource=move || judge_priv.get()
                view= move |judge| view! {
                <a href="#" role="button" class="outline" on:click=move |_| {
                    accept.dispatch(PostRequest {
                        data: NominationRequest {user: judge.user, prediction: judge.prediction},
                        access: state.get().access.unwrap()});
                    set_count.set(count.get() + 1);
                }>
                    "Accept"
                </a>
                <a href="#" role="button" class="outline contrast" on:click=move |_| {
                    refuse.dispatch(PostRequest {
                        data: NominationRequest {user: judge.user, prediction: judge.prediction},
                        access: state.get().access.unwrap()});
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
            move || match bets.get() {
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
                let name = usernames.get().transpose().ok().flatten().unwrap_or_default();
                if name.is_empty() {
                    user.to_string().into_view()
                } else {
                    view! {<span title={user.to_string()} >{name}</span>}.into_view()
                }
            } else {
                "User".to_string().into_view()
            }
        }
    }}
}
#[component]
pub fn MyBets(access: ReadSignal<MercadoState>) -> impl IntoView {
    let bets = create_local_resource(move || access.get().access, my_bets);

    view! {
        <table>
            <tr>
                <th>"Bet"</th>
                <th>"Amount"</th>
                <th>"Prediction"</th>
                <th>"State"</th>
                <th>"Actions"</th>
            </tr>
            <For each=move || bets.get().transpose().ok().flatten().unwrap_or_default() key=move |bet| bet.user
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
#[component]
pub fn NewPrediction(state: ReadSignal<MercadoState>) -> impl IntoView {
    let (prediction, set_prediction) = create_signal(String::from("This works"));
    let (end, set_end) = create_signal(String::from(
        (Utc::now() + Duration::days(5)).to_rfc3339().split_at(16).0,
    ));
    let (judge_count, set_judge_count) = create_signal("3".to_string());
    let (judges, set_judges) = create_signal::<Vec<UserPubKey>>(vec![]);
    let (new_judge, set_new_judge) =
        create_signal(generate_keypair(&mut rand::thread_rng()).1.to_string());
    let (decision, set_decision) = create_signal("3".to_string());
    let (judge_share, set_judge_share) = create_signal("10000".to_string());

    let parsed_end = move || (end.get() + ":00Z").parse::<DateTime<Utc>>();
    let display_end = move || match parsed_end() {
        Ok(parsed) => parsed.to_string().into_view(),
        Err(_e) => "".into_view(),
    };

    let (create_message, set_create_message) = create_signal(view! {}.into_view());

    view! {
        <div>
            <h3>"Create a new prediction"</h3>
            <label>"Prediction"</label>
            <input type="text"
                on:input=move |e| {
                    set_prediction.set(event_target_value(&e));
                }
                value={prediction}
            />
            <div class="grid">
                <label>"Ends at "{display_end}
                <input type="datetime-local" on:input=move |e| { set_end.set(event_target_value(&e)); } value={end}/></label>
                <label>"Days of decision period for judges"
                <input type="number" on:input=move |e| { set_decision.set(event_target_value(&e))} value={decision} /></label>
            </div>
            <div class="grid">
                <label>"Portion for Judges (ppm)"
                <input type="number" on:input=move |e| { set_judge_share.set(event_target_value(&e))} value={judge_share} /></label>
                <label>"How many judges need to participate?"
                <input type="number" on:input=move |e| { set_judge_count.set(event_target_value(&e)) } value={judge_count}/></label>
            </div>
            <p>"Judges: "
                <input type="text" on:input=move |e| {set_new_judge.set(event_target_value(&e)) } value={new_judge}/>
            <a href="#" role="button" on:click=move |_| {
                if let Ok(judge) = new_judge.get().parse() {
                    let mut judges = judges.get();
                    judges.push(judge);
                    set_judges.set(judges);
                    set_new_judge.set(generate_keypair(&mut rand::thread_rng()).1.to_string());
                } else {
                    return
                }
            }>"Add"</a></p>
            <ul>
            <For each=move || judges.get() key=move |judge| judge.clone()
                view=move |judge: UserPubKey| view!{
                    <li>{judge.to_string()}" "
                        <a href="#" role="button" class="contrast"
                            on:click=move |_| {
                                let mut judges = judges.get();
                                judges.retain(|judge_item| {
                                    judge_item != &judge
                                });
                                set_judges.set(judges);
                            }
                        >"Remove"</a>
                    </li>
                } />
            </ul>
            {move || create_message.get()}
            <button on:click=move |_| {
                let result = move || {
                    let request = NewPredictionRequest {
                        decision_period_sec: decision.get().parse::<u32>()? * 86400,
                        judge_count: judge_count.get().parse()?,
                        judge_share_ppm: judge_share.get().parse()?,
                        judges: judges.get(),
                        prediction: prediction.get(),
                        trading_end: parsed_end()?
                    };
                    let new_prediction_id = create_action(|request: &NewPredictionRequest| new_prediction(request.clone()));
                    new_prediction_id.dispatch(request);
                    match new_prediction_id.value().get() {
                        Some(Ok(rowid)) => {
                            Ok(rowid)
                        }
                        Some(Err(e)) => {
                            bail!("{:?}", e)
                        }
                        None => {
                            bail!("Got NONE from action")
                        }
                    }
                };
                match result() {
                    Ok(rowid) => {
                        //FIXME this branch doesn't get called because we only get NONE from the
                        //action
                        let navigate = leptos_router::use_navigate();
                        navigate(format!("/prediction/{}", rowid).as_str(), Default::default());
                    }
                    Err(e) => {
                        set_create_message.set(view!{
                            <label>{format!("{:?}", e)}</label>
                        }.into_view());
                    }
                }
            } >"Create"</button>
        </div>
    }
}
#[component]
pub fn AddBet(state: ReadSignal<MercadoState>) -> impl IntoView {
    let predictions = create_local_resource(move || {}, get_predictions);
    let (search, set_search) = create_signal(String::new());
    let (bet, set_bet) = create_signal(String::new());
    let (amount, set_amount) = create_signal(String::new());
    let is_admin = if let Some(user) = state.get().user {
        user.role == UserRole::Admin || user.role == UserRole::Root
    } else {
        false
    };
    view! {
        <div>
            <h3>"New bet"</h3>
            <label>"Search predictions"<input type="search" on:input=move |e| {set_search.set(event_target_value(&e))}/></label>
            <label>"Prediction"
                <select>
                    <option disabled value="" selected>"Select a prediction"</option>
                    <For each=move || {
                        match predictions.get() {
                            Some(Ok(mut predictions)) => {
                                predictions.retain(|prediction| {
                                    if let Ok(id) = search.get().parse::<i64>() {
                                        prediction.id == id
                                    } else {
                                        prediction.name.contains(search.get().as_str())
                                    }
                                });
                                predictions
                            },
                            None | Some(Err(_))=> vec![],
                        }
                    }
                    key=move |prediction| prediction.id
                    view=move |prediction| {
                        view! {
                            <option>{prediction.name}" ("{prediction.id}")"</option>
                        }
                    }
                    />

                </select>
            </label>
            <div class="grid">
                <fieldset>
                    <legend>"Bet"</legend>
                    <label>
                    <input type="radio" value="true" name="bet" on:input=move |e| {set_bet.set(event_target_value(&e))} />
                    "True"
                    </label>
                    <label>
                    <input type="radio" value="false" name="bet" on:input=move |e| {set_bet.set(event_target_value(&e))} />
                    "False"
                    </label>
                </fieldset>
                {
                    if is_admin {
                        view!{
                            <label>"Amount (sats)"
                            <input type="number" on:input=move |e| {set_amount.set(event_target_value(&e))} />
                            </label>
                        }.into_view()
                    } else {view!{}.into_view()}
                }
            </div>
            <div>
            <a href="" role="button" on:click=move |_| {} >"Add"</a>" "
            {
                if is_admin {
                    view!{
                        <a href="" role="button" on:click=move |_| {} >"Add & Pay"</a>
                    }.into_view()
                } else {view!{}.into_view()}
            }
            </div>
        </div>
    }
}
