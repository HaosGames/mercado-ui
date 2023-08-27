use crate::{components::*, fetchers::check_login};
use leptos::*;
use leptos_router::*;
use mercado::api::AccessRequest;

mod components;
mod fetchers;

const URL: &str = "http://127.0.0.1:8081";
const STORAGE_KEY: &str = "mercado_access";

fn main() {
    mount_to_body(|| {
        let (access, set_access) = create_signal::<Option<AccessRequest>>(None);
        let storage_access = window().local_storage().ok().flatten().and_then(|storage| {
            storage
                .get_item(STORAGE_KEY)
                .ok()
                .flatten()
                .and_then(|value| serde_json::from_str::<Option<AccessRequest>>(&value).ok())
                .unwrap()
        });
        set_access.set(storage_access);
        create_effect(move |_| {
            if let Ok(Some(storage)) = window().local_storage() {
                let json =
                    serde_json::to_string(&access.get()).expect("couldn't serialize AccessRequest");
                if storage.set_item(STORAGE_KEY, &json).is_err() {
                    error!("error while trying to set item in localStorage");
                }
            }
        });
        let check_login = create_local_resource(move || access.get().unwrap(), check_login);

        view! {
            <div id="root">
                <Router>
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
                                            <li><a>"Bets"</a></li>
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
                    <main class="container">
                        <Routes>
                            <Route path="" view=App/>
                            <Route path="prediction" view=move || view! {<Outlet/>}>
                                <Route path=":id" view=move || view! {<PredictionOverview access=access />}/>
                                <Route path="" view=App/>
                            </Route>
                            <Route path="login" view=move || view! {<Login set_access=set_access />} />
                        </Routes>
                    </main>
                </Router>
            </div>
        }
    })
}
