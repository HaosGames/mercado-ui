#![allow(non_snake_case)]
use crate::{fetchers::*, MercadoState};
use anyhow::{bail, Context};
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
                <li><a href="/"><strong>"Mercado"</strong></a></li>
                <li>
                    <details role="list" >
                        <summary aria-haspopup="listbox" role="link" >"New"</summary>
                        <ul role="listbox">
                            <li><A href="/new_prediction">"Prediction"</A></li>
                            <li><A href="/add_bet">"Bet"</A></li>
                        </ul>
                    </details>
                </li>
            </ul>
            <ul>
                {move || {
                    let access = if let Some(access) = state.get().access {
                        access
                    } else {
                        return view! {}.into_view();
                    };
                    let balances = create_local_resource(move || {}, move |_| get_balances_for(access.clone()));
                    view!{
                        <UnwrapResourceFor state=state resource=balances view=move |balances| { view! {
                            <li>{balances.0}"/"{balances.1}" sats"</li>
                        }} />
                    }.into_view()
                }}
                <li>{
                    move || if state.get().access.is_some() && check_login.get().transpose().ok().flatten().is_some() {
                        view!{
                            <details role="list" >
                                <summary aria-haspopup="listbox" role="link" ><Username user={
                                    if let Some(access) = state.get().access {
                                        Some(access.user)
                                    } else {
                                        None
                                    }
                                } no_clipboard=true /></summary>
                                <ul role="listbox">
                                    <li><a>"Edit user"</a></li>
                                    <li><a>"Predictions"</a></li>
                                    <li><a href="/my_bets">"Bets"</a></li>
                                    <li><a href="/my_judges">"Judges"</a></li>
                                    <li><a href="/" on:click=move |_| {set_state.set(MercadoState::default())} >"Logout"</a></li>
                                </ul>
                            </details>
                        }.into_view()
                    } else {
                        view!{
                            <a href="/login">"Login"</a>
                        }.into_view()
                    }
                }</li>
            </ul>
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
pub fn PredictionListItem(
    prediction: PredictionOverviewResponse,
    refresh: RwSignal<bool>,
) -> impl IntoView {
    let ratio = create_local_resource(
        move || refresh.get(),
        move |_| {
            get_prediction_ratio(PredictionRequest {
                prediction: prediction.id,
                user: None,
            })
        },
    );
    view! {
        <tr>
            <td><a href={format!("/prediction/{}", prediction.id)}>{prediction.name}</a></td>
            <td>{prediction.trading_end.to_string()}</td>
            <td>{prediction.judge_share_ppm / 10000}"%"</td>
            <td>{prediction.state.to_string()}</td>
            <td><UnwrapResource resource=ratio view=move |ratio| view! {
                    <span>{format!("True: {}% ({} sats)",
                         ratio.0 as f32/(ratio.0+ratio.1)as f32*100.0,
                         ratio.0,
                    )}</span>
                    <span style="float:right">{format!("False: {}% ({} sats)",
                         ratio.1 as f32/(ratio.0+ratio.1)as f32*100.0,
                         ratio.1,
                    )}</span><br/>
                    <progress value={ratio.0} max={ratio.0+ratio.1}>"Ratio"</progress>
                } /></td>
        </tr>
    }
}
#[component]
pub fn PredictionList() -> impl IntoView {
    let predictions = create_local_resource(move || {}, get_predictions);
    let refresh = create_rw_signal(true);

    view! {
        <UnwrapResource resource=predictions view=move |mut predictions| view! {
            <p>{predictions.len()}" prediction(s)"
                <span style="float:right">
                    <a href="" role="button" on:click=move |_| refresh.set(!refresh.get())>"Refresh"</a>
                </span>
            </p>
            <table role="grid">
                <tr>
                   <th>"Prediction"</th>
                   <th>"End"</th>
                   <th>"Judge Share"</th>
                   <th>"State"</th>
                   <th>"Ratio"</th>
                </tr>
                {
                    predictions.sort_by(|a, b| a.id.cmp(&b.id));
                    predictions.into_iter()
                    .map(|prediction| view! {<PredictionListItem prediction=prediction refresh=refresh/>})
                    .collect::<Vec<_>>()
                }
            </table>
        } />
    }
}
#[component]
pub fn UnwrapResource<F, V, T, S>(
    view: F,
    resource: Resource<S, Result<T, String>>,
) -> impl IntoView
where
    F: Fn(T) -> V + 'static,
    V: IntoView,
    T: Clone + 'static,
    S: Clone + 'static,
{
    view! {
        {
            move || match resource.get() {
                None => view! {<small aria-busy="true">"Loading..."</small>}.into_view(),
                Some(Ok(t)) => view(t).into_view(),
                Some(Err(e)) => view! {<small>{format!("{}", e)}</small>}.into_view(),
            }
        }
    }
}
#[component]
pub fn UnwrapResourceForUser<F, V, T, S>(
    view: F,
    resource: Resource<S, Result<T, String>>,
    user: UserPubKey,
    state: ReadSignal<MercadoState>,
) -> impl IntoView
where
    F: Fn(T) -> V + 'static,
    V: IntoView,
    T: Clone + 'static,
    S: Clone + 'static,
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
pub fn UnwrapResourceFor<F, V, T, S>(
    view: F,
    resource: Resource<S, Result<T, String>>,
    state: ReadSignal<MercadoState>,
) -> impl IntoView
where
    F: Fn(T) -> V + 'static,
    V: IntoView,
    T: Clone + 'static,
    S: Clone + 'static,
{
    if let Some(access) = state.get_untracked().access {
        view! {
            <UnwrapResourceForUser view=view resource=resource state=state user=access.user />
        }
        .into_view()
    } else {
        "".into_view()
    }
}
#[component]
pub fn PredictionOverview(state: ReadSignal<MercadoState>) -> impl IntoView {
    let params = use_params_map();
    let id = params.with_untracked(|p| p.get("id").cloned());
    let id = id.unwrap_or_default().parse::<RowId>().unwrap();
    let refresh = create_rw_signal(true);
    let prediction =
        create_local_resource(move || refresh.get(), move |_| get_prediction_overview(id));
    let ratio = create_local_resource(
        move || refresh.get(),
        move |_| {
            get_prediction_ratio(PredictionRequest {
                prediction: id,
                user: None,
            })
        },
    );
    let force_decision_period =
        create_action(move |&()| force_decision_period(id, state.get().access.unwrap()));
    let user = if let Some(user) = state.get_untracked().user {
        if let UserRole::User = user.role {
            Some(user.user)
        } else {
            None
        }
    } else {
        None
    };
    view! {
        <UnwrapResource resource=prediction view=move |prediction| view! {
            <h3>{prediction.name.clone()}</h3>
            {
                if let Some(user) = state.get().user {
                    if prediction.state == MarketState::Trading
                        && (user.role == UserRole::Root || user.role == UserRole::Admin)
                    { view!{
                            <span style="float:right"><a href="" role="button"
                                on:click=move |_| {
                                    force_decision_period.dispatch(());
                                    refresh.set(!refresh.get());
                                }
                            >"Force decision period"</a></span>
                        }.into_view()
                    } else {view!{}.into_view()}
                } else {view!{}.into_view()}
            }
            <p>
                "State: "<kbd>{prediction.state.to_string()}</kbd><br/>
                "End: "{prediction.trading_end.to_string()}<br/>
                "Time left: "{
                        let left = (prediction.trading_end - Utc::now());
                        format!("{} weeks {} days {} hours",
                            left.num_weeks(),
                            left.num_days() % 7,
                            left.num_hours() % 24
                        )
                    }<br/>
                "Judge share: "{prediction.judge_share_ppm as f32/10000.0}"%"<br/>
                "Judges: "{prediction.judge_count}<br/>
                "Decision period: "{prediction.decision_period_sec/86400}" days"
            </p>
            <p>
                <UnwrapResource resource=ratio view=move |ratio| view! {
                    <span>{format!("True: {}% ({} sats)",
                         ratio.0 as f32/(ratio.0+ratio.1)as f32*100.0,
                         ratio.0,
                    )}</span>
                    <span style="float:right">{format!("False: {}% ({} sats)",
                         ratio.1 as f32/(ratio.0+ratio.1)as f32*100.0,
                         ratio.1,
                    )}</span><br/>
                    <progress value={ratio.0} max={ratio.0+ratio.1}>"Ratio"</progress><br/>
                    <p style="text-align:center">"Total: "{ratio.0+ratio.1}" sats"</p>
                } />
                <a href="" role="button" on:click=move |_| {
                    refresh.set(!refresh.get());
                }>"Refresh"</a>
            </p>
            <JudgeList prediction=Some(prediction.id) user=user state=state refresh=refresh />
            <BetList prediction=Some(prediction.id) state=state collapsable=true
                user=user
            />
            <p>"Id: "{prediction.id}</p>
        } />
    }
}
#[component]
pub fn JudgeList(
    prediction: Option<RowId>,
    user: Option<UserPubKey>,
    state: ReadSignal<MercadoState>,
    #[prop(optional)] refresh: Option<RwSignal<bool>>,
) -> impl IntoView {
    let refresh = if let Some(refresh) = refresh {
        refresh
    } else {
        create_rw_signal(true)
    };
    let judges = create_local_resource(
        move || prediction.clone(),
        move |prediction| get_judges(prediction, user),
    );
    view! {
        <UnwrapResource
            resource=judges
            view=move |judges| view! {
            <details open>
                <summary>{format!("Judges: {}", judges.len())}</summary>
                <table>
                    <tr>
                        <th>"Judge"</th>
                        <th>"State"</th>
                        <th>"Actions"</th>
                    </tr>
                    <For each=move || judges.clone() key=move |judge| judge.user
                    children=move |judge: JudgePublic| view!{
                        <JudgeListItem judge=judge state=state refresh=refresh />
                    }/>
                </table>
            </details>
        } />
    }
}
#[component]
pub fn JudgeListItem(
    judge: JudgePublic,
    state: ReadSignal<MercadoState>,
    #[prop(optional)] refresh: Option<RwSignal<bool>>,
) -> impl IntoView {
    let refresh = if let Some(refresh) = refresh {
        refresh
    } else {
        create_rw_signal(true)
    };
    let prediction = create_local_resource(
        move || (judge.prediction, refresh.get()),
        move |(id, _)| get_prediction_overview(id),
    );
    let accept = create_action(|request: &PostRequest<NominationRequest>| {
        accept_nomination(request.data.clone(), request.access.clone())
    });
    let refuse = create_action(|request: &PostRequest<NominationRequest>| {
        refuse_nomination(request.data.clone(), request.access.clone())
    });
    let decide = create_action(move |(judge, bet): &(Judge, bool)| {
        make_decision(
            MakeDecisionRequest {
                decision: bet.clone(),
                judge: judge.user,
                prediction: judge.prediction.clone(),
            },
            state.get().access.unwrap(),
        )
    });
    let judge_priv = create_local_resource(
        move || refresh.get(),
        move |_| get_judge(judge.prediction, judge.user, state),
    );
    view! {
        <tr>
            <td><Username user= Some(judge.user) /></td>
            <td><UnwrapResourceForUser
                user=judge.user
                state=state
                resource=judge_priv
                view=move |judge| judge.state.to_string()
            /></td>
            <td><UnwrapResourceForUser
            user=judge.user
            state=state
            resource=judge_priv
            view= move |judge| {
                match prediction.get().transpose().ok().flatten().map(|prediction| prediction.state).unwrap_or(MarketState::Trading) {
                    MarketState::WaitingForJudges => {
                        view! {
                            <a href="" role="button" class="outline" on:click=move |_| {
                                accept.dispatch(PostRequest {
                                    data: NominationRequest {user: judge.user, prediction: judge.prediction},
                                    access: state.get().access.unwrap()});
                                refresh.set(!refresh.get());
                            }>
                                "Accept"
                            </a>
                            <a href="" role="button" class="outline contrast" on:click=move |_| {
                                refuse.dispatch(PostRequest {
                                    data: NominationRequest {user: judge.user, prediction: judge.prediction},
                                    access: state.get().access.unwrap()});
                                refresh.set(!refresh.get());
                            }>
                                "Refuse"
                            </a>
                        }.into_view()
                    }
                    MarketState::WaitingForDecision => {
                        view! {
                            <a href="" role="button" class="outline" on:click=move |_| {
                                decide.dispatch((judge, true));
                                refresh.set(!refresh.get());
                            }>
                                "Decide True"
                            </a>
                            <a href="" role="button" class="outline contrast" on:click=move |_| {
                                decide.dispatch((judge, false));
                                refresh.set(!refresh.get());
                            }>
                                "Decide False"
                            </a>

                        }.into_view()
                    }
                    _ => {view!{}.into_view()}
                }
                } />
            </td>
        </tr>
    }
}
#[component]
pub fn Cond<V>(cond: bool, view: V) -> impl IntoView
where
    V: IntoView,
{
    if cond {
        view.into_view()
    } else {
        view! {}.into_view()
    }
}
#[component]
pub fn BetList(
    prediction: Option<RowId>,
    user: Option<UserPubKey>,
    state: ReadSignal<MercadoState>,
    #[prop(optional)] collapsable: Option<bool>,
) -> impl IntoView {
    let access = if let Some(access) = state.get_untracked().access {
        access
    } else {
        return view! {}.into_view();
    };
    let bets = create_local_resource(
        move || PredictionUserRequest { prediction, user },
        move |request| get_bets(request, access.clone()),
    );
    let table = move |bets: Vec<Bet>| {
        view! {
            <table>
                <tr>
                    <th>"Bet"</th>
                    <th>"Amount"</th>
                    <Cond cond=user.is_none() view=view!{<th>"User"</th>}/>
                    <Cond cond=prediction.is_none() view=view!{<th>"Prediction"</th>}/>
                </tr>
                <For each=move || bets.clone() key=move |judge| judge.user
                children=move |bet: Bet| view!{
                    <tr>
                        <td>{bet.bet}</td>
                        <td>{bet.amount}</td>
                        <Cond cond=user.is_none() view=view!{<td><Username user=Some(bet.user) /></td>}/>
                        <Cond cond=prediction.is_none() view=view!{<td><a href={format!("/prediction/{}", bet.prediction)}>"Prediction"</a></td>}/>
                    </tr>
                }/>
            </table>
        }
    };
    if let Some(true) = collapsable {
        view! {
            <UnwrapResourceFor state=state resource=bets view=move |bets| view!{
                <details>
                    <summary>{format!("Bets: {}", bets.len())}</summary>
                    {table(bets)}
                </details>
            } />
        }
    } else {
        view! {
            <UnwrapResourceFor state=state resource=bets view=move |bets| view!{
                {table(bets)}
            } />
        }
    }
}
#[component]
pub fn ShortenedString(
    mut string: String,
    #[prop(optional)] no_clipboard: Option<bool>,
) -> impl IntoView {
    let open = create_rw_signal(false);
    let original = string.clone();
    let end = string.split_off(59);
    string.truncate(8);
    string = string + "..." + end.as_str();
    view! {
        <span>
            <small>{string}</small>
            {
            if let None |Some(false) = no_clipboard {
                view!{
                    <a href="" on:click=move |_| {
                        // TODO Enable copying to clipboard
                        // if let Some(clipboard) = window().navigator().clipboard() {
                        //     clipboard.write_text(original.as_str());
                        // }
                        open.set(true);
                    }>"📋 "</a>
                    <dialog open=open>
                        <article>
                            <header>
                                <a href="" class="close" aria-label="Close" on:click=move |_| open.set(false)></a>
                                "Copy"
                            </header>
                            <p>{original}</p>
                        </article>
                    </dialog>
                }.into_view()
            } else {view!{}.into_view()}
            }
        </span>
    }
    .into_view()
}
#[component]
pub fn Username(
    user: Option<UserPubKey>,
    #[prop(optional)] no_clipboard: Option<bool>,
) -> impl IntoView {
    let usernames = create_local_resource(move || user.unwrap(), get_username);
    view! {{
        move || {
            if let Some(user) = user {
                let name = usernames.get().transpose().ok().flatten().unwrap_or_default();
                if name.is_empty() {
                    let mut user = user.to_string();
                    if let None | Some(false) = no_clipboard {
                        view!{<ShortenedString string=user />}.into_view()
                    } else {
                        view!{<ShortenedString string=user no_clipboard=true />}.into_view()
                    }
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
pub fn MyBets(state: ReadSignal<MercadoState>) -> impl IntoView {
    let user = if let Some(user) = state.get().user {
        if let UserRole::User = user.role {
            Some(user.user)
        } else {
            None
        }
    } else {
        None
    };

    view! {
        <BetList state=state prediction=None user=user />
    }
}
#[component]
pub fn MyJudges(state: ReadSignal<MercadoState>) -> impl IntoView {
    let user = if let Some(user) = state.get().user {
        if let UserRole::User = user.role {
            Some(user.user)
        } else {
            None
        }
    } else {
        None
    };

    view! {
        <JudgeList state=state prediction=None user=user />
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

    let new_prediction_action =
        create_action(|request: &NewPredictionRequest| new_prediction(request.clone()));
    let message = create_rw_signal(Some(view! {}.into_view()));
    let new_prediction_id = create_local_resource(
        move || new_prediction_action.version().get(),
        move |_| fetch_rw_signal(new_prediction_action.value()),
    );

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
                children=move |judge: UserPubKey| view!{
                    <li><ShortenedString string=judge.to_string() />" "
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
            {
                move || {
                    if let Some(message) = message.get() {
                        message
                    } else {
                        match new_prediction_id.get().flatten() {
                            Some(Ok(rowid)) => {
                                view!{<Redirect path={format!("/prediction/{}", rowid)} />}.into_view()
                            }
                            Some(Err(e)) => {
                                view!{<label>{format!("{:?}", e)}</label>}.into_view()
                            }
                            None => {
                                view!{}.into_view()
                            }
                        }
                    }
                }
            }
            <button on:click=move |_| {
                let result = move || {
                    let request = NewPredictionRequest {
                        decision_period_sec: decision.get().parse::<u32>().context("Decision period needs to be a number")? * 86400,
                        judge_count: judge_count.get().parse().context("Judge count needs to be a number")?,
                        judge_share_ppm: judge_share.get().parse().context("Judge share needs to be a number")?,
                        judges: judges.get(),
                        prediction: prediction.get(),
                        trading_end: parsed_end().context("Trading end needs to be in a valid format")?
                    };
                    new_prediction_action.dispatch(request);
                    Ok::<(),anyhow::Error>(())
                };
                match result() {
                    Ok(_) => {
                        message.set(None)
                    }
                    Err(e) => {
                        message.set(Some(view!{
                            <label>{format!("{}", e)}</label>
                        }.into_view()));
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
    let (prediction, set_prediction) = create_signal(String::new());
    let (bet, set_bet) = create_signal(String::new());
    let (amount, set_amount) = create_signal(String::from("100"));
    let message = create_rw_signal(None);

    let create_new_bet = create_action(|(request, access): &(AddBetRequest, AccessRequest)| {
        add_bet(request.clone(), access.clone())
    });
    let add_bet = move || {
        let access = if let Some(access) = state.get().access {
            access
        } else {
            bail!("Not logged in")
        };
        let request = AddBetRequest {
            prediction: prediction.get().parse().context("Choose a prediction")?,
            bet: bet.get().parse().context("Choose True or False")?,
            user: access.user,
            amount: amount.get().parse().context("Chose a valid amount")?,
        };
        create_new_bet.dispatch((request, access));
        Ok(())
    };
    let created_bet = create_local_resource(
        move || create_new_bet.version().get(),
        move |_| fetch_rw_signal(create_new_bet.value()),
    );
    view! {
        <div>
            <h3>"New bet"</h3>
            <label>"Filter predictions"
                <input type="search" placeholder="Name or ID" autofocus on:input=move |e| {set_search.set(event_target_value(&e))}/>
            </label>
            <label>"Prediction "{move || prediction.get()}
                <select on:input=move |e| {set_prediction.set(event_target_value(&e))}>
                    <option value="" selected disabled >"Select prediction"</option>
                    <For each=move || {
                        match predictions.get() {
                            Some(Ok(mut predictions)) => {
                                predictions.retain(|prediction| {
                                    if prediction.state != MarketState::Trading {return false}
                                    if let Ok(id) = search.get().parse::<i64>() {
                                        prediction.id == id
                                    } else {
                                        prediction.name.contains(search.get().as_str())
                                    }
                                });
                                predictions.sort_by(|a,b| a.name.cmp(&b.name));
                                predictions
                            },
                            None | Some(Err(_))=> vec![],
                        }
                    }
                    key=move |prediction| prediction.id
                    children=move |prediction| {
                        view! {
                            <option value={prediction.id}>{prediction.name}" ("{prediction.id}")"</option>
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
                <label>
                <input type="number" value=amount on:input=move |e| {set_amount.set(event_target_value(&e))} />
                "Amount"
                </label>
            </div>
            <label><small>
            {
                move || {
                    if let Some(message) = message.get() {
                        message
                    } else {
                        match created_bet.get().flatten() {
                            Some(Ok(payment)) => {
                                //TODO redirect to bet status page to enable paying
                                view!{<Redirect path={format!("/")} />}.into_view()
                            }
                            Some(Err(e)) => {
                                format!("{:?}", e).into_view()
                            }
                            None => {
                                view!{}.into_view()
                            }
                        }
                    }
                }
            }</small>
            <button on:click=move |_| {
                match add_bet() {
                    Ok(action) => message.set(None),
                    Err(e) => message.set(Some(e.to_string().into_view())),
                }
            } >"Add"</button></label>
        </div>
    }
}
