use {
    yew::prelude::*,
    yew_router::prelude::*,
    serde::Deserialize,
    gloo_storage::{LocalStorage, Storage},
    crate::utils::Route,
};

#[derive(Deserialize, Debug)]
struct LoginQuery {
    token: String,
}

#[function_component(LoginPage)]
pub fn login_page() -> Html {
    let navigator = use_navigator().unwrap();

    let location = match use_location() {
        Some(v) => v,
        None => return html!(<div>{"error: no location"}</div>),
    };
    let query: LoginQuery = location.query().unwrap();

    LocalStorage::set("access_token", query.token).unwrap();
    navigator.push(&Route::Home);

    html!(
        <div>
            {"Login"}
        </div>
    )
}