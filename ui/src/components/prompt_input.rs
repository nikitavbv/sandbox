use {
    yew::prelude::*,
    stylist::{style, yew::styled_component},
    web_sys::{EventTarget, HtmlInputElement},
    wasm_bindgen::JsCast,
};

#[derive(Properties, PartialEq)]
pub struct PromptInputProps {
    pub value: String,

    pub on_change: Callback<String>,
    pub on_run_inference: Callback<()>,
}

#[styled_component(PromptInput)]
pub fn prompt_input(props: &PromptInputProps) -> Html {
    let input_style = style!(r#"
        padding: 8px;
        font-size: 14pt;
        border-radius: 5px;
        border: 2px solid white;
        outline: none;
        width: 400px;
        font-family: Inter;
        transition: border-color 0.2s ease-out;
        background-color: white;
        color: black;
  
        :focus {
            border: 2px solid #5695DC;
        }
    "#).unwrap();

    let generate_image_button_style = style!(r#"
        margin-left: 8px;
        padding: 8px 14px;
        font-size: 14pt;
        background-color: #5695DC;
        color: white;
        border: 2px solid #5695DC;
        border-radius: 5px;
        font-family: Inter;
        cursor: pointer;
        width: 192px;
        user-select: none;
        transition:
            color 0.2s ease-out, 
            background-color 0.2s ease-out;

        :hover {
            background-color: #F6F4F3;
            color: #5695DC;
        }
    "#).unwrap();

    html!(
        <>
            <input 
                class={input_style} 
                onchange={
                    let on_change = props.on_change.clone();

                    move |e: Event| {
                        let target: Option<EventTarget> = e.target();
                        let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
                        if let Some(input) = input {
                            on_change.emit(input.value());
                        }
                    }
                } 
                value={props.value.clone()} 
                placeholder={"prompt, for example: cute cat"} />
            <button 
                class={generate_image_button_style} 
                onclick={
                    let on_run_inference = props.on_run_inference.clone();
                    move |_| on_run_inference.emit(())
                }>
                {"generate image"}
                </button>
        </>
    )
}