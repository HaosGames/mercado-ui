use crate::components::*;
use leptos::*;
use leptos_router::*;

mod components;

fn main() {
    mount_to_body(|cx| {
        view! { cx,
            <div id="root">
                <Router>
                    <nav>
                        <a href="/">"Home"</a>
                    </nav>
                    <main>
                        <Routes>
                            <Route path="" view=App/>
                            <Route path="prediction" view=move |_| view! {cx, <Outlet/>}>
                                <Route path=":id" view=UserPredictionOverview/>
                                <Route path="" view=App/>
                            </Route>
                        </Routes>
                    </main>
                </Router>
            </div>
        }
    })
}
