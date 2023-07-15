use {
    stylist::{style, yew::styled_component},
    yew::prelude::*,
    yew_router::prelude::*,
    crate::utils::Route,
};

#[styled_component(Header)]
pub fn header() -> Html {
    let style = style!(r#"
        width: 100%;
        height: 50px;
        background-color: #292929;
        font-size: 14pt;
        color: white;
        line-height: 50px;
        padding-left: 24px;
    "#).unwrap();

    let title_style = style!(r#"
        cursor: pointer;
    "#).unwrap();

    let navigator = use_navigator().unwrap();
    let return_home = Callback::from(move |_| {
        navigator.push(&Route::Home);
    });

    html!(
        <header class={style}>
            <span class={title_style} onclick={return_home}>{ "sandbox" }</span>
        </header>
    )
}