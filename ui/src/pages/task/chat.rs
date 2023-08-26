use {
    yew::prelude::*,
    stylist::{style, yew::styled_component},
};

#[styled_component(ChatMessageGenerationTask)]
pub fn chat() -> Html {
    html!("chat")
}