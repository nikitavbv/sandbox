use {
    yew::prelude::*,
    yew_router::prelude::*,
    tracing::info,
    serde::Deserialize,
};

#[derive(Deserialize, Debug)]
struct LoginQuery {
    token: String,
}

#[function_component(LoginPage)]
pub fn login_page() -> Html {
    let location = match use_location() {
        Some(v) => v,
        None => return html!(<div>{"error: no location"}</div>),
    };
    let query: LoginQuery = location.query().unwrap();

    info!("location: {:?}", query);

    html!(
        <div>
            {"Login"}
        </div>
    )
}