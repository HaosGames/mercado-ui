use crate::components::*;
use leptos::*;
use leptos_meta::*;
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
                .unwrap_or(None)
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

        view! {
            <div id="root">
                <Stylesheet href="https://cdn.jsdelivr.net/npm/@picocss/pico@1/css/pico.min.css" />
                <Router>
                    <Navi access=access set_access=set_access />
                    <main class="container">
                        <Routes>
                            <Route path="" view=App/>
                            <Route path="prediction" view=move || view! {<Outlet/>}>
                                <Route path=":id" view=move || view! {<PredictionOverview access=access />}/>
                                <Route path="" view=App/>
                            </Route>
                            <Route path="login" view=move || view! {<Login set_access=set_access />} />
                            <Route path="my_bets" view=move || view!{<MyBets access=access />} />
                        </Routes>
                    </main>
                </Router>
            </div>
        }
    })
}
