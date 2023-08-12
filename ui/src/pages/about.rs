use {
    yew::prelude::*,
    stylist::{style, yew::styled_component},
};

#[styled_component(AboutPage)]
pub fn about_page() -> Html {
    let readme = include_str!("../../../README.md");
    
    let li_style = style!(r#"
        margin-bottom: 2px;

        ::before {
            content: "- ";
        }
    "#).unwrap();

    let features = {
        let features_header = "# Features";
        let features_index = readme.find(features_header).unwrap();
        let todos_index = readme.find("# TODOs").unwrap();

        readme[(features_index + features_header.len())..todos_index].to_owned()
            .lines()
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
            .map(|v| html!(<li class={li_style.clone()}>{&v[v.find("- ").unwrap()+2..]}</li>))
            .collect::<Vec<_>>()
    };

    let todos = {
        let todos_header = "# TODOs";
        let todos_index = readme.find(todos_header).unwrap();
        let acknowledgments_index = readme.find("# Acknowledgments").unwrap();

        readme[(todos_index + todos_header.len())..acknowledgments_index].to_owned()
            .lines()
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
            .map(|v| html!(<li class={li_style.clone()}>{&v[v.find("- ").unwrap()+2..]}</li>))
            .collect::<Vec<_>>()
    };

    let page_style = style!(r#"
        max-width: 900px;
        margin: 0 auto;
        line-height: 1.5;
    "#).unwrap();

    let header_style = style!(r#"
        font-size: 18pt;
        margin: 16px 0 4px 0;
    "#).unwrap();

    let link_style: stylist::Style = style!(r#"
        color: #90caf9;
        text-decoration: none;
    "#).unwrap();

    html!(
        <div class={page_style}>
            <h1 class={header_style.clone()}>{"sandbox: web app for exploring generative ai models"}</h1>
            {"This web app is built for learning and fun purposes. All components are written in Rust. Source code is available on "}
            <a class={link_style.clone()} href={"https://github.com/nikitavbv/sandbox"}>{"Github"}</a>{"."}

            <h1 class={header_style.clone()}>{"Usage"}</h1>
            { "You can either use this instance or host your own (it is not as simple as `cargo run --release` yet, but close to that)." }

            <h1 class={header_style.clone()}>{"Features"}</h1>
            <ul>{ features }</ul>

            <h1 class={header_style.clone()}>{"TODOs"}</h1>
            <ul>{ todos }</ul>

            <h1 class={header_style.clone()}>{"Acknowledgments"}</h1>
            {"Most of the heavy lifting is performed by "}
            <a class={link_style.clone()} href={"https://github.com/huggingface/candle"}>{"candle"}</a>
            {" (which is an amazing library) and code samples from candle examples."}
        </div>
    )
}
