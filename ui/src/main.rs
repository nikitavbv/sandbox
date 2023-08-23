use {
    std::{sync::{Arc, Mutex}, rc::Rc},
    yew::prelude::*,
    yew_router::prelude::*,
    wasm_bindgen_futures::spawn_local,
    tracing_wasm::WASMLayerConfigBuilder,
    web_sys::{EventTarget, HtmlInputElement},
    wasm_bindgen::JsCast,
    gloo_storage::{Storage, LocalStorage},
    stylist::{style, yew::styled_component},
    crate::{
        components::header::Header,
        pages::{
            task::TaskPage,
            login::LoginPage,
            history::HistoryPage,
            home::HomePage,
            about::AboutPage,
        },
        utils::{client_with_token, Route},
    },
};

pub mod components;
pub mod pages;
pub mod utils;

#[styled_component(App)]
fn app() -> Html {
    html!(
        <div>
            <BrowserRouter>
                <Switch<Route> render={router_switch} />
            </BrowserRouter>
        </div>
    )
}

fn router_switch(route: Route) -> Html {
    html!(<RouterComponent route={route} />)
}

#[derive(Properties, PartialEq)]
struct RouterComponentProps {
    route: Route,
}

#[function_component(RouterComponent)]
fn router_component(props: &RouterComponentProps) -> Html {
    let style = style!(r#"
        padding: 24px;
    "#).unwrap();

    let is_logged_in = use_state(|| LocalStorage::get::<String>("access_token").is_ok());
    let logout = {
        let is_logged_in_setter = is_logged_in.setter();
        
        move |_| {
            LocalStorage::delete("access_token");
            is_logged_in_setter.set(false);
        }
    };
    let login = {
        let is_logged_in_setter = is_logged_in.setter();

        move |_: ()| {
            is_logged_in_setter.set(true);
        }
    };

    let body = match &props.route {
        Route::Home => html!(<HomePage />),
        Route::Login => html!(<LoginPage login={login} />),
        Route::Task { id }=> html!(<TaskPage task_id={id.clone()} />),
        Route::History => html!(<HistoryPage />),
        Route::About => html!(<AboutPage />),
    };

    let header_component = if &Route::Login == &props.route {
        html!()
    } else {
        html!(<Header current_route={props.route.clone()} is_logged_in={*is_logged_in} logout={logout} />)
    };

    html!(
        <>
            { header_component }      
            <main class={style}>
                { body }
            </main>
        </>
    )
}

fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default_with_config(
        WASMLayerConfigBuilder::new()
            .set_max_level(tracing::Level::INFO)
            .build()
        );
    yew::Renderer::<App>::new().render();
}
