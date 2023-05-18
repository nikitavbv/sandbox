use {
    yew::prelude::*,
    yew_router::prelude::*,
    crate::utils::Route,
};

#[function_component(HistoryPage)]
pub fn history_page() -> Html {
    let navigator = use_navigator().unwrap();
    
    let return_home = Callback::from(move |_| {
        navigator.push(&Route::Home);
    });

    html!(
        <div>
            <button onclick={return_home}>{"home"}</button>
            <h1>{"history"}</h1>
        </div>
    )
}