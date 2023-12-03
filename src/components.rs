#![allow(non_snake_case)]
use crate::{fetchers::*, MercadoState};
use anyhow::{bail, Context};
use chrono::DateTime;
use chrono::{offset::Utc, Duration};
use leptonic::prelude::*;
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
        <AppBar>
            <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(1.0) style="padding-left: 20px">
                <Link href="/"><H2>"Mercado"</H2></Link>
                <ButtonGroup>
                    <LinkButton href="/new_prediction">"New Prediction"</LinkButton>
                    <LinkButton href="/add_bet">"New Bet"</LinkButton>
                </ButtonGroup>
            </Stack>
            <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(1.0) style="padding-right: 20px">
                <Link href="/">"Predictions"</Link>
                <Link href="/my_bets">"Bets"</Link>
                <Link href="/my_judges">"Judges"</Link>
                <Link href="#">"Users"</Link>
                {move || {
                    let access = if let Some(access) = state.get().access {
                        access
                    } else {
                        return view! {}.into_view();
                    };
                    let balances = create_local_resource(move || {}, move |_| get_balances_for(access.clone()));
                    view!{
                        <UnwrapResourceFor state=state resource=balances view=move |balances| { view! {
                            <Link href="/wallet">{balances.0}"/"{balances.1}" sats"</Link>
                        }} />
                    }.into_view()
                }}
                {
                    move || if state.get().access.is_some() && check_login.get().transpose().ok().flatten().is_some() {
                        view!{
                            <Username user={
                                if let Some(access) = state.get().access {
                                    Some(access.user)
                                } else {
                                    None
                                }
                            } no_clipboard=true />
                            <LinkButton href="/" on:click=move |_| {set_state.set(MercadoState::default())} >"Logout"</LinkButton>
                        }.into_view()
                    } else {
                        view!{
                            <LinkButton href="/login">"Login"</LinkButton>
                        }.into_view()
                    }
                }
            <ThemeToggle off=LeptonicTheme::Light on=LeptonicTheme::Dark/>
            </Stack>
        </AppBar>
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
        <Grid spacing=Size::Em(0.6)>
            <Row>
                <Col><Link href={format!("/prediction/{}", prediction.id)}>{prediction.name}</Link></Col>
            </Row>
            <Row>
                <Col xs=1>{prediction.trading_end.to_string()}</Col>
                <Col xs=1>{prediction.judge_share_ppm / 10000}"%"</Col>
                <Col xs=1>{prediction.state.to_string()}</Col>
                <Col xs=1><UnwrapResource resource=ratio view=move |ratio| view! {
                    <Stack spacing=Size::Zero style="width:100%">
                        <Stack orientation=StackOrientation::Horizontal spacing=Size::Px(8)>
                        <span>{format!("True: {}% ({} sats)",
                             ratio.0 as f32/(ratio.0+ratio.1)as f32*100.0,
                             ratio.0,
                        )}</span>
                        <span style="float:right">{format!("False: {}% ({} sats)",
                             ratio.1 as f32/(ratio.0+ratio.1)as f32*100.0,
                             ratio.1,
                        )}</span></Stack>
                        <ProgressBar progress=Some(ratio.0 as f64) max={(ratio.0+ratio.1) as f64} />
                    </Stack>
                } /></Col>
            </Row>
        </Grid>
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
                    <Button on_click=move |_| refresh.set(!refresh.get())>"Refresh"</Button>
                </span>
            </p>
            <Grid spacing=Size::Em(0.6)><Row>
                <Col xs=1><H3>"Trading End"</H3></Col>
                <Col xs=1><H3>"Judge Share"</H3></Col>
                <Col xs=1><H3>"State"</H3></Col>
                <Col xs=1><H3>"Capital"</H3></Col>
            </Row></Grid>
            <Stack spacing=Size::Em(1.0)>
                {
                    predictions.sort_by(|a, b| a.id.cmp(&b.id));
                    predictions.into_iter()
                    .map(|prediction| view! {<PredictionListItem prediction=prediction refresh=refresh/>})
                    .collect::<Vec<_>>()
                }
            </Stack>
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
                    <ProgressBar progress={Some(ratio.0 as f64)} max={(ratio.0+ratio.1) as f64}/>
                    <p style="text-align:center">"Total: "{ratio.0+ratio.1}" sats"</p>
                } />
                <Button on_click=move |_| {
                    refresh.set(!refresh.get());
                }>"Refresh"</Button>
            </p>
            <JudgeList prediction=Some(prediction.id) user=user state=state refresh=refresh collapsable=true/>
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
    #[prop(optional)] collapsable: Option<bool>,
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
    let table = move |judges: Vec<JudgePublic>| {
        view! {
            <TableContainer><Table bordered=true hoverable=true>
                <Thead><Tr>
                    <Th>"Judge"</Th>
                    <Th>"State"</Th>
                    <Th>"Actions"</Th>
                </Tr></Thead>
                <Tbody><For each=move || judges.clone() key=move |judge| judge.user
                children=move |judge: JudgePublic| view!{
                    <JudgeListItem judge=judge state=state refresh=refresh />
                }/></Tbody>
            </Table></TableContainer>
        }
    };
    if let Some(true) = collapsable {
        view! {
            <UnwrapResourceFor state=state resource=judges view=move |judges| {
                let judgec = judges.clone();
                view!{
                <Collapsible>
                    <CollapsibleHeader slot>{format!("Judges: {}", judgec.len())}</CollapsibleHeader>
                    <CollapsibleBody slot>{table(judges)}</CollapsibleBody>
                </Collapsible>
            }} />
        }
    } else {
        view! {
            <UnwrapResourceFor state=state resource=judges view=move |judges| view!{
                {table(judges)}
            } />
        }
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
        <Tr>
            <Td><Username user= Some(judge.user) /></Td>
            <Td><UnwrapResourceForUser
                user=judge.user
                state=state
                resource=judge_priv
                view=move |judge| judge.state.to_string()
            /></Td>
            <Td><UnwrapResourceForUser
            user=judge.user
            state=state
            resource=judge_priv
            view= move |judge| {
                match prediction.get().transpose().ok().flatten().map(|prediction| prediction.state).unwrap_or(MarketState::Trading) {
                    MarketState::WaitingForJudges => {
                        view! {
                            <Button on_click=move |_| {
                                accept.dispatch(PostRequest {
                                    data: NominationRequest {user: judge.user, prediction: judge.prediction},
                                    access: state.get().access.unwrap()});
                                refresh.set(!refresh.get());
                            }>
                                "Accept"
                            </Button>
                            <Button on_click=move |_| {
                                refuse.dispatch(PostRequest {
                                    data: NominationRequest {user: judge.user, prediction: judge.prediction},
                                    access: state.get().access.unwrap()});
                                refresh.set(!refresh.get());
                            }>
                                "Refuse"
                            </Button>
                        }.into_view()
                    }
                    MarketState::WaitingForDecision => {
                        view! {
                            <Button on_click=move |_| {
                                decide.dispatch((judge, true));
                                refresh.set(!refresh.get());
                            }>
                                "Decide True"
                            </Button>
                            <Button on_click=move |_| {
                                decide.dispatch((judge, false));
                                refresh.set(!refresh.get());
                            }>
                                "Decide False"
                            </Button>

                        }.into_view()
                    }
                    _ => {view!{}.into_view()}
                }
                } />
            </Td>
        </Tr>
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
    #[prop(optional)] refresh: Option<RwSignal<bool>>,
) -> impl IntoView {
    let access = if let Some(access) = state.get_untracked().access {
        create_rw_signal(access)
    } else {
        return view! {}.into_view();
    };
    let refresh = if let Some(refresh) = refresh {
        refresh
    } else {
        create_rw_signal(true)
    };
    let bets = create_local_resource(
        move || (PredictionUserRequest { prediction, user }, refresh.get()),
        move |(request, _)| get_bets(request, access.get()),
    );
    let cancel_bet = create_action(|(id, access): &(RowId, AccessRequest)| {
        cancel_bet(id.clone(), access.clone())
    });
    let table = move |bets: Vec<Bet>| {
        view! {
            <TableContainer><Table bordered=true hoverable=true>
                <Thead><Tr>
                    <Th>"Bet"</Th>
                    <Th>"Amount"</Th>
                    <Cond cond=user.is_none() view=view!{<Th>"User"</Th>}/>
                    <Cond cond=prediction.is_none() view=view!{<Th>"Prediction"</Th>}/>
                    <Th>"Actions"</Th>
                </Tr></Thead>
                <Tbody><For each=move || bets.clone() key=move |bet| bet.user
                children=move |bet: Bet| view!{
                    <Tr>
                        <Td>{bet.bet}</Td>
                        <Td>{bet.amount}</Td>
                        <Cond cond=user.is_none() view=view!{<Td><Username user=Some(bet.user) /></Td>}/>
                        <Cond cond=prediction.is_none() view=view!{<Td><Link href={format!("/prediction/{}", bet.prediction)}>"Prediction"</Link></Td>}/>
                        <Td><Button on_click=move |_| {
                            cancel_bet.dispatch((bet.id, access.get()));
                            refresh.set(!refresh.get());
                        }>"Cancel"</Button></Td>
                    </Tr>
                }/></Tbody>
            </Table></TableContainer>
        }
    };
    if let Some(true) = collapsable {
        view! {
            <UnwrapResourceFor state=state resource=bets view=move |bets| {
                let betc = bets.clone();
                view!{
                <Collapsible>
                    <CollapsibleHeader slot>{format!("Bets: {}", betc.len())}</CollapsibleHeader>
                    <CollapsibleBody slot>{table(bets)}</CollapsibleBody>
                </Collapsible>
            }} />
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
    let side_length = 8;
    let open = create_rw_signal(false);
    let original = string.clone();
    let end = string.split_off(string.len() - side_length);
    string.truncate(side_length);
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
                    <Modal show_when=open>
                        <ModalHeader>"Copy"</ModalHeader>
                        <ModalTitle>""</ModalTitle>
                        <ModalBody><p><small>{original}</small></p></ModalBody>
                        <ModalFooter><Button on_click=move |_| open.set(false)>"Close"</Button></ModalFooter>
                    </Modal>
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
    let (end, set_end) = create_signal(time::OffsetDateTime::now_utc());
    let (judge_count, set_judge_count) = create_signal(3.0);
    let (judges, set_judges) = create_signal::<Vec<UserPubKey>>(vec![]);
    let (new_judge, set_new_judge) =
        create_signal(generate_keypair(&mut rand::thread_rng()).1.to_string());
    let (decision, set_decision) = create_signal(3.0);
    let (judge_share, set_judge_share) = create_signal(10000.0);

    let new_prediction_action =
        create_action(|request: &NewPredictionRequest| new_prediction(request.clone()));
    let message = create_rw_signal(Some(view! {}.into_view()));
    let new_prediction_id = create_local_resource(
        move || new_prediction_action.version().get(),
        move |_| fetch_rw_signal(new_prediction_action.value()),
    );

    view! {
        <Stack spacing=Size::Em(1.0)>
            <H3>"Create a new prediction"</H3>
            <Box style="width: 50%"><TextInput get=prediction set=set_prediction placeholder="Prediction" /></Box>
            <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(2.0) >
                <div>
                    <DateSelector value=time::OffsetDateTime::now_utc() on_change=move |date: time::OffsetDateTime| {
                        set_end.set(date)
                    } />
                    <label>"Ends at "{move || format!("{}", end.get())}</label>
                </div>
                <div>
                    <div>
                        <NumberInput get=decision set=set_decision step=1.0 />
                        <label>"Decision duration"</label>
                    </div>
                    <div>
                        <NumberInput get=judge_share set=set_judge_share step=1000.0 />
                        <label>"Portion for Judges (ppm)"</label>
                    </div>
                    <div>
                        <NumberInput get=judge_count set=set_judge_count step=1.0 />
                        <label>"How many judges need to participate?"</label>
                    </div>
                </div>
            </Stack>
            <Box style="width: 50%">"Judges: "
                <TextInput get=new_judge set=set_new_judge placeholder="Judge Public Key" />
            <Button on_click=move |_| {
                if let Ok(judge) = new_judge.get().parse() {
                    let mut judges = judges.get();
                    judges.push(judge);
                    set_judges.set(judges);
                    set_new_judge.set(generate_keypair(&mut rand::thread_rng()).1.to_string());
                } else {
                    return
                }
            }>"Add"</Button></Box>
            <ul>
            <For each=move || judges.get() key=move |judge| judge.clone()
                children=move |judge: UserPubKey| view!{
                    <li><ShortenedString string=judge.to_string() />" "
                        <Button
                            on_click=move |_| {
                                let mut judges = judges.get();
                                judges.retain(|judge_item| {
                                    judge_item != &judge
                                });
                                set_judges.set(judges);
                            }
                        >"Remove"</Button>
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
            <Button on_click=move |_| {
                let result = move || {
                    let request = NewPredictionRequest {
                        decision_period_sec: decision.get() as u32 * 86400,
                        judge_count: judge_count.get() as u32,
                        judge_share_ppm: judge_share.get() as u32,
                        judges: judges.get(),
                        prediction: prediction.get(),
                        trading_end: DateTime::from_timestamp(end.get().unix_timestamp(),0).unwrap(),
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
            } >"Create"</Button>
        </Stack>
    }
}
#[component]
pub fn AddBet(state: ReadSignal<MercadoState>) -> impl IntoView {
    let predictions = create_local_resource(move || {}, get_predictions);
    let (search, set_search) = create_signal(String::new());
    let (prediction, set_prediction) = create_signal::<Option<PredictionOverviewResponse>>(None);
    let (bet, set_bet) = create_signal(false);
    let (amount, set_amount) = create_signal(100.0);
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
            prediction: prediction.get().unwrap().id,
            bet: bet.get(),
            user: access.user,
            amount: amount.get() as Sats,
        };
        create_new_bet.dispatch((request, access));
        Ok(())
    };
    let created_bet = create_local_resource(
        move || create_new_bet.version().get(),
        move |_| fetch_rw_signal(create_new_bet.value()),
    );
    view! {
        <Stack spacing=Size::Em(1.0)>
            <h3>"New bet"</h3>
            <Box style="width: 50%"><UnwrapResource resource=predictions view=move |mut predictions|
            {
                predictions.retain(|prediction| {
                    if prediction.state != MarketState::Trading {return false}
                    if let Ok(id) = search.get().parse::<i64>() {
                        prediction.id == id
                    } else {
                        prediction.name.contains(search.get().as_str())
                    }
                });
                predictions.sort_by(|a,b| a.name.cmp(&b.name));
                view!{
                    <OptionalSelect options=predictions
                        selected=prediction
                        set_selected=move |v| set_prediction.set(v)
                        search_text_provider=move |o: PredictionOverviewResponse| format!("{} ({})", o.name, o.id)
                        render_option=move |o: PredictionOverviewResponse| format!("{} ({})", o.name, o.id)
                        allow_deselect=false
                    />
                }

            } /></Box>
            <div>
                <Toggle state=bet set_state=set_bet icons=ToggleIcons {
                    on: leptos_icons::BsIcon::BsCheck.into(),
                    off: leptos_icons::BsIcon::BsX.into(),
                } style="--toggle-slider-on-background-color: green; --toggle-slider-off-background-color: red;"/>
                <p>{move || format!("Bet: {}", bet.get())}</p>
            </div>
            <div>
                <NumberInput get=amount set=set_amount step=100.0 />
                <label>"Amount (sats)"</label>
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
            <Button on_click=move |_| {
                match add_bet() {
                    Ok(action) => message.set(None),
                    Err(e) => message.set(Some(e.to_string().into_view())),
                }
            } >"Add"</Button></label>
        </Stack>
    }
}
#[component]
pub fn Wallet(state: ReadSignal<MercadoState>) -> impl IntoView {
    let access = if let Some(access) = state.get_untracked().access {
        create_rw_signal(access)
    } else {
        return view! {}.into_view();
    };
    let params = use_params_map();
    let id = params.with_untracked(|p| p.get("id").cloned());
    let user = id.unwrap_or_default().parse::<UserPubKey>();
    let user = if let Ok(user) = user {
        user
    } else {
        return view! {<Redirect path={format!("/wallet/{}", access.get().user.to_string())}/>}
            .into_view();
    };
    let balances = create_local_resource(
        move || (user, access.get()),
        move |(user, a)| get_balances_for_user(user, a),
    );
    let deposits = create_local_resource(
        move || (user, access.get()),
        move |(user, a)| {
            let request = TxsRequest {
                user: Some(user),
                direction: Some(TxDirection::Deposit),
            };
            get_txs(request, a)
        },
    );
    let withdrawals = create_local_resource(
        move || (user, access.get()),
        move |(user, a)| {
            let request = TxsRequest {
                user: Some(user),
                direction: Some(TxDirection::Withdrawal),
            };
            get_txs(request, a)
        },
    );

    view! {
        <Stack spacing=Size::Em(1.0)>
            <H3>"Bitcoin Wallet"</H3>
            <UnwrapResourceFor state=state resource=balances view=move |balances| view!{
                <p>
                    "Available Balance: "{balances.0}" sats"<br/>
                    "Total Balance: "{balances.1}" sats"<br/>
                </p>
            } />
            <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(2.0)>
                <div>
                    <LinkButton href="/make_deposit">"Make Deposit"</LinkButton>
                    <UnwrapResourceForUser user=user state=state resource=deposits view=move |deposits| view!{
                        <TableContainer><Table bordered=true hoverable=true>
                            <Thead><tr>
                                <th>"Invoice"</th>
                                <th>"State"</th>
                            </tr></Thead>
                            <Tbody><For each=move || deposits.clone() key=|id| id.clone() children=move |id: RowId| view!{
                                <DepositListItem state=state id=id />
                            } /></Tbody>
                        </Table></TableContainer>
                    } />
                </div>
                <div>
                    <LinkButton href="/make_withdrawal">"Make Withdrawal"</LinkButton>
                    <UnwrapResourceForUser user=user state=state resource=withdrawals view=move |withdrawals| view!{
                        <TableContainer><Table bordered=true hoverable=true>
                            <Thead><tr>
                                <th>"Payment hash"</th>
                                <th>"State"</th>
                            </tr></Thead>
                            <Tbody><For each=move || withdrawals.clone() key=|id| id.clone() children=move |id: RowId| view!{
                                <WithdrawListItem state=state id=id />
                            } /></Tbody>
                        </Table></TableContainer>
                    } />
                </div>
            </Stack>
        </Stack>
    }
    .into_view()
}
#[component]
pub fn DepositListItem(state: ReadSignal<MercadoState>, id: RowId) -> impl IntoView {
    let access = if let Some(access) = state.get_untracked().access {
        create_rw_signal(access)
    } else {
        return view! {}.into_view();
    };
    let tx = create_local_resource(move || id, move |id| get_tx(id, access.get()));

    view! {
        <tr>
            <UnwrapResourceFor state=state resource=tx view=move |tx| {
                match tx.tx_type {
                    TxType::Bolt11 {details, state} => {
                        view!{
                            <td><ShortenedString string={details.payment_request}/></td>
                            <td>{format!("{:?}", state)}</td>
                        }
                    }
                }
            } />
        </tr>
    }
    .into_view()
}
#[component]
pub fn WithdrawListItem(state: ReadSignal<MercadoState>, id: RowId) -> impl IntoView {
    let access = if let Some(access) = state.get_untracked().access {
        create_rw_signal(access)
    } else {
        return view! {}.into_view();
    };
    let tx = create_local_resource(move || id, move |id| get_tx(id, access.get()));

    view! {
        <tr>
            <UnwrapResourceFor state=state resource=tx view=move |tx| {
                match tx.tx_type {
                    TxType::Bolt11 {details, state} => {
                        view!{
                            <td><ShortenedString string={details.payment_hash}/></td>
                            <td>{format!("{:?}", state)}</td>
                        }
                    }
                }
            } />
        </tr>
    }
    .into_view()
}
#[component]
pub fn MakeDeposit(state: ReadSignal<MercadoState>) -> impl IntoView {
    let access = if let Some(access) = state.get_untracked().access {
        access
    } else {
        return view! {}.into_view();
    };
    let params = use_params_map();
    let id = params.with_untracked(|p| p.get("id").cloned());
    let user = id
        .unwrap_or_default()
        .parse::<UserPubKey>()
        .unwrap_or(access.user);
    let amount = create_rw_signal(1000.0);
    let tx_type = create_rw_signal(String::from("bolt11"));

    let make_new_deposit = create_action(|(request, access): &(DepositRequest, AccessRequest)| {
        make_deposit_bolt11(request.clone(), access.clone())
    });
    let add_bet = move || {
        let access = if let Some(access) = state.get().access {
            access
        } else {
            bail!("Not logged in")
        };
        let request = DepositRequest {
            amount: amount.get() as Sats,
            user,
        };
        make_new_deposit.dispatch((request, access));
        Ok(())
    };
    let created_deposit = create_local_resource(
        move || make_new_deposit.version().get(),
        move |_| fetch_rw_signal(make_new_deposit.value()),
    );
    let message = create_rw_signal(None);

    view! {
        <Stack spacing=Size::Em(1.0)>
            <h3>"Make Deposit"</h3>
            <div>
                <NumberInput get=amount set=amount.write_only() step=1000.0 />
                <label>"Amount"</label>
            </div>
            <p>{
                move || {
                    if let Some(message) = message.get() {
                        message
                    } else {
                        match created_deposit.get().flatten() {
                            Some(Ok((_id, invoice))) => {
                                //TODO redirect to bet status page to enable paying
                                invoice.into_view()
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
            }</p>
            <Button on_click=move |_| {
                match add_bet() {
                    Ok(action) => message.set(None),
                    Err(e) => message.set(Some(e.to_string().into_view())),
                }
            } >"Get Payment Details"</Button>
        </Stack>
    }
    .into_view()
}
#[component]
pub fn MakeWithdrawal(state: ReadSignal<MercadoState>) -> impl IntoView {
    let access = if let Some(access) = state.get_untracked().access {
        access
    } else {
        return view! {}.into_view();
    };
    let params = use_params_map();
    let id = params.with_untracked(|p| p.get("id").cloned());
    let user = id
        .unwrap_or_default()
        .parse::<UserPubKey>()
        .unwrap_or(access.user);
    let amount = create_rw_signal(1000.0);
    let invoice = create_rw_signal(String::from(""));
    let tx_type = create_rw_signal(String::from("bolt11"));

    let make_new_withdrawal =
        create_action(|(request, access): &(WithdrawalRequest, AccessRequest)| {
            make_withdrawal_bolt11(request.clone(), access.clone())
        });
    let add_bet = move || {
        let access = if let Some(access) = state.get().access {
            access
        } else {
            bail!("Not logged in")
        };
        let request = WithdrawalRequest {
            amount: amount.get() as Sats,
            invoice: invoice.get(),
            user,
        };
        make_new_withdrawal.dispatch((request, access));
        Ok(())
    };
    let created_withdrawal = create_local_resource(
        move || make_new_withdrawal.version().get(),
        move |_| fetch_rw_signal(make_new_withdrawal.value()),
    );
    let message = create_rw_signal(None);

    view! {
        <Stack spacing=Size::Em(1.0)>
            <h3>"Make Withdrawal"</h3>
            <div>
                <NumberInput get=amount set=amount.write_only() step=1000.0 />
                <label>"Amount (sats)"</label>
            </div>
            <TextInput get=invoice set=invoice.write_only() placeholder="Invoice" />
            <p>{
                move || {
                    if let Some(message) = message.get() {
                        message
                    } else {
                        match created_withdrawal.get().flatten() {
                            Some(Ok(id)) => {
                                view!{<Redirect path={format!("/wallet")} />}.into_view()
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
            }</p>
            <Button on_click=move |_| {
                match add_bet() {
                    Ok(action) => message.set(None),
                    Err(e) => message.set(Some(e.to_string().into_view())),
                }
            } >"Withdraw"</Button>
        </Stack>
    }
    .into_view()
}
