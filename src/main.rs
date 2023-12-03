#![allow(unused)]
use crate::components::*;
use leptonic::prelude::*;
use leptos::*;
use leptos_router::*;
use mercado::api::{AccessRequest, UserResponse};
use serde::{Deserialize, Serialize};

mod components;
mod fetchers;

const URL: &str = "http://127.0.0.1:8081";
const STORAGE_KEY: &str = "mercado-state";

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct MercadoState {
    access: Option<AccessRequest>,
    user: Option<UserResponse>,
}

fn main() {
    mount_to_body(|| {
        let (state, set_state) = create_signal::<MercadoState>(MercadoState {
            access: None,
            user: None,
        });
        let storage_access = window()
            .local_storage()
            .ok()
            .flatten()
            .and_then(|storage| {
                storage
                    .get_item(STORAGE_KEY)
                    .ok()
                    .flatten()
                    .and_then(|value| serde_json::from_str::<MercadoState>(&value).ok())
            })
            .unwrap_or_default();
        set_state.set(storage_access);
        create_effect(move |_| {
            if let Ok(Some(storage)) = window().local_storage() {
                let json =
                    serde_json::to_string(&state.get()).expect("couldn't serialize AccessRequest");
                if storage.set_item(STORAGE_KEY, &json).is_err() {
                    log::error!("error while trying to set item in localStorage");
                }
            }
        });

        view! {
            <Root default_theme=LeptonicTheme::Dark>
                <Router>
                    <Navi state=state set_state=set_state />
                    <Box style="padding-left: 20px; padding-right: 20px;"><Routes>
                        <Route path="" view=App/>
                        <Route path="prediction" view=move || view! {<Outlet/>}>
                            <Route path=":id" view=move || view! {<PredictionOverview state=state />}/>
                            <Route path="" view=App/>
                        </Route>
                        <Route path="login" view=move || view! {<Login set_state=set_state />} />
                        <Route path="my_bets" view=move || view!{<MyBets state=state />} />
                        <Route path="my_judges" view=move || view!{<MyJudges state=state />} />
                        <Route path="new_prediction" view=move || view!{<NewPrediction state=state />} />
                        <Route path="add_bet" view=move || view!{<AddBet state=state />} />
                        <Route path="wallet" view=move || view! {<Outlet/>}>
                            <Route path=":id" view=move || view! {<Wallet state=state />}/>
                            <Route path="" view=move || view!{<Wallet state=state/>}/>
                        </Route>
                        <Route path="make_deposit" view=move || view! {<Outlet/>}>
                            <Route path=":id" view=move || view! {<MakeDeposit state=state />}/>
                            <Route path="" view=move || view!{<MakeDeposit state=state/>}/>
                        </Route>
                        <Route path="make_withdrawal" view=move || view! {<Outlet/>}>
                            <Route path=":id" view=move || view! {<MakeWithdrawal state=state />}/>
                            <Route path="" view=move || view!{<MakeWithdrawal state=state/>}/>
                        </Route>
                    </Routes></Box>
                </Router>
            </Root>
        }
    })
}
