#![allow(unused)]
use crate::components::*;
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
            <div id="root">
                <Router>
                    <Navi state=state set_state=set_state />
                    <main class="container">
                        <Routes>
                            <Route path="" view=App/>
                            <Route path="prediction" view=move || view! {<Outlet/>}>
                                <Route path=":id" view=move || view! {<PredictionOverview state=state />}/>
                                <Route path="" view=App/>
                            </Route>
                            <Route path="login" view=move || view! {<Login set_state=set_state />} />
                            <Route path="my_bets" view=move || view!{<MyBets access=state />} />
                            <Route path="new_prediction" view=move || view!{<NewPrediction state=state />} />
                            <Route path="add_bet" view=move || view!{<AddBet state=state />} />
                            <Route path="cash_outs" view=move || view!{<MyCashOuts state=state />} />
                        </Routes>
                    </main>
                </Router>
            </div>
        }
    })
}
