use {
    std::sync::{Arc, Mutex},
    yew::prelude::*,
    yew_router::prelude::*,
    serde::Deserialize,
    gloo_storage::{LocalStorage, Storage},
    wasm_bindgen_futures::spawn_local,
    web_sys::window,
    rpc::OAuthLoginRequest,
    crate::utils::{Route, client},
};

#[derive(Deserialize, Debug)]
struct LoginQuery {
    code: String,
}

#[derive(Properties, PartialEq)]
pub struct LoginProps {
    pub login: Callback<()>,
}

#[function_component(LoginPage)]
pub fn login_page(props: &LoginProps) -> Html {
    let navigator = use_navigator().unwrap();

    let client = Arc::new(Mutex::new(client()));

    let location = match use_location() {
        Some(v) => v,
        None => return html!(<div>{"error: no location"}</div>),
    };
    let query: LoginQuery = location.query().unwrap();

    let login_callback = props.login.clone();
    use_effect_with_deps(move |code| {
        let code = code.clone();

        spawn_local(async move {
            let mut client = client.lock().unwrap();

            let res = client.o_auth_login(OAuthLoginRequest {
                code: code.to_owned(),
                redirect_uri: format!("{}/login", window().unwrap().location().origin().unwrap()),
            }).await.unwrap().into_inner();

            LocalStorage::set("access_token", res.token).unwrap();
            login_callback.emit(());
            navigator.push(&Route::Home);
        });
    }, query.code.clone());

    html!(
        <div>
            {"please wait..."}
        </div>
    )
}