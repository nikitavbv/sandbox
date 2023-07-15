use {
    stylist::{style, yew::styled_component},
    yew::prelude::*,
    yew_router::prelude::*,
    gloo_storage::{Storage, LocalStorage},
    crate::utils::{Route, MultiClass},
};

#[styled_component(Header)]
pub fn header() -> Html {
    let style = style!(r#"
        width: 100%;
        height: 50px;
        background-color: #292929;
        font-size: 14pt;
        line-height: 50px;
        padding: 0 24px;
        user-select: none;
        font-weight: 400;
    "#).unwrap();

    let title_style = style!(r#"
        cursor: pointer;
        color: white;
        padding-right: 24px;
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
    "#).unwrap();

    let navigator = use_navigator().unwrap();
    let is_logged_in = LocalStorage::get::<String>("access_token").is_ok();

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

    let history_menu_entry = if is_logged_in {
        html!(<span class={menu_entry_style.clone()} onclick={open_history}>{"history"}</span>)
    } else {
        html!()
    };

    html!(
        <header class={style}>
            <span class={title_style} onclick={return_home}>{ "sandbox" }</span>
            { history_menu_entry }
            <span class={MultiClass::new().with(&menu_entry_style).with(&login_ctl_style)}>{"login"}</span>
        </header>
    )
}