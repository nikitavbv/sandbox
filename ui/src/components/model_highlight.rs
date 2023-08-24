use {
    yew::prelude::*,
    stylist::{style, yew::styled_component},
};

#[derive(Properties, PartialEq)]
pub struct ModelHighlightProps {
    pub children: Children,
}

#[styled_component(ModelHighlight)]
pub fn model_highlight(props: &ModelHighlightProps) -> Html {
    let description_style = style!(r#"
        width: 100%;
        text-align: center;
        user-select: none;
        padding-bottom: 16px;
        display: block;
        font-size: 16pt;
        line-height: 24pt;
        color: white;
    "#).unwrap();

    html!(
        <span class={description_style}>{ props.children.clone() }</span>
    )
}