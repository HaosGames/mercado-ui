use crate::components::*;
use leptos::*;
use leptos_router::*;

mod components;
mod fetchers;

const URL: &str = "http://127.0.0.1:8081";

fn main() {
    mount_to_body(|| {
        view! {
            <div id="root">
                <Router>
                    <nav>
                        <a href="/">"Home"</a>
                    </nav>
                    <main>
                        <Routes>
                            <Route path="" view=App/>
                            <Route path="prediction" view=move || view! {<Outlet/>}>
                                <Route path=":id" view=PredictionOverview/>
                                <Route path="" view=App/>
                            </Route>
                        </Routes>
                    </main>
                </Router>
            </div>
        }
    })
}
