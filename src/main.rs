use crate::components::*;
use leptos::*;

mod components;

fn main() {
    mount_to_body(|cx| view! { cx,  <App/> })
}

