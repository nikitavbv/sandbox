use {
    std::collections::HashMap,
    stylist::{style, yew::styled_component},
    yew::prelude::*,
    yew_router::prelude::*,
    gloo_storage::{Storage, LocalStorage},
    web_sys::window,
    crate::utils::{Route, MultiClass, start_oauth_flow},
};

#[derive(Properties, PartialEq)]
pub struct HeaderProps {
    pub current_route: Route,
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
        transition: background-color 0.2s ease-out;

        :hover {
            background-color: #3D3D3D;
        }
    "#).unwrap();

    let active_menu_entry_style = style!(r#"
        border-bottom: 2px solid #CED0CE;
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
        let style = MultiClass::new().with(&menu_entry_style);
        let style = if props.current_route == Route::History {
            style.with(&active_menu_entry_style)
        } else {
            style
        };

        html!(<span class={style} onclick={open_history}>{"tasks"}</span>)
    } else {
        html!()
    };

    let about_menu_entry = {
        let style = MultiClass::new().with(&menu_entry_style);
        let style = if props.current_route == Route::About {
            style.with(&active_menu_entry_style)
        } else {
            style
        };
        
        let open_about = {
            let navigator = navigator.clone();
            Callback::from(move |_| {
                navigator.push(&Route::About);
            })
        };

        html!(<span class={style} onclick={open_about}>{"about"}</span>)
    };

    let login = Callback::from(move |_| start_oauth_flow());

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
            { about_menu_entry }
            { login_menu_entry }
            { logout_menu_entry }
        </header>
    )
}