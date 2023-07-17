use {
    std::collections::HashMap,
    stylist::{style, yew::styled_component},
    yew::prelude::*,
    yew_router::prelude::*,
    gloo_storage::{Storage, LocalStorage},
    web_sys::window,
    crate::utils::{Route, MultiClass},
};

#[derive(Properties, PartialEq)]
pub struct HeaderProps {
    pub is_logged_in: bool,
    pub logout: Callback<()>,
}

#[styled_component(Header)]
pub fn header(props: &HeaderProps) -> Html {
    let style = style!(r#"
        width: 100%;
        height: 50px;
        background-color: #292929;
        font-size: 14pt;
        line-height: 50px;
        user-select: none;
        font-weight: 400;
    "#).unwrap();

    let title_style = style!(r#"
        cursor: pointer;
        color: white;
        padding: 0 24px;
    "#).unwrap();

    let menu_entry_style = style!(r#"
        padding: 0 12px;
        font-size: 12pt;
        cursor: pointer;
        height: 100%;
        display: inline-block;

        :hover {
            background-color: #3D3D3D;
        }
    "#).unwrap();

    let login_ctl_style = style!(r#"
        float: right;
        padding: 0 24px;
    "#).unwrap();

    let navigator = use_navigator().unwrap();

    let return_home = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::Home);
        })
    };

    let open_history = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::History);
        })
    };

    let history_menu_entry = if props.is_logged_in {
        html!(<span class={menu_entry_style.clone()} onclick={open_history}>{"tasks"}</span>)
    } else {
        html!()
    };

    let login = Callback::from(move |_| {
        let redirect_uri = format!("{}/login", window().unwrap().location().origin().unwrap());

        let mut query_params = HashMap::new();
        query_params.insert("client_id", "916750455653-biu6q4c7llj7q1k14h3qaquktcdlkeo4.apps.googleusercontent.com".to_owned());
        query_params.insert("response_type", "code".to_owned());
        query_params.insert("scope", "https://www.googleapis.com/auth/userinfo.profile https://www.googleapis.com/auth/userinfo.email".to_owned()); 

        let query_string = form_urlencoded::Serializer::new("".to_owned())
            .extend_pairs(query_params.iter())
            .finish();

        window().unwrap().location().set_href(&format!("https://accounts.google.com/o/oauth2/v2/auth?redirect_uri={}&{}", redirect_uri, query_string)).unwrap();
    });

    let login_menu_entry = if props.is_logged_in {
        html!()
    } else {
        html!(<span class={MultiClass::new().with(&menu_entry_style).with(&login_ctl_style)} onclick={login}>{"login"}</span>)
    };

    let logout = props.logout.clone();

    let logout_menu_entry = if props.is_logged_in {
        html!(<span class={MultiClass::new().with(&menu_entry_style).with(&login_ctl_style)} onclick={move |_| logout.emit(())}>{"logout"}</span>)
    } else {
        html!()
    };

    html!(
        <header class={style}>
            <span class={title_style} onclick={return_home}>{ "sandbox" }</span>
            { history_menu_entry }
            { login_menu_entry }
            { logout_menu_entry }
        </header>
    )
}