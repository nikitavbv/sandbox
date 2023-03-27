use {
    tracing::info,
    rust_bert::pipelines::summarization::SummarizationModel,
    crate::models::{io::ModelData, Model},
};

pub struct TextSummarizationModel {
    model: SummarizationModel,
}

impl TextSummarizationModel {
    pub fn new() -> Self {
        Self {
            model: SummarizationModel::new(Default::default()).unwrap(),
        }
    }
}

impl Model for TextSummarizationModel {
    fn run(&self, input: &ModelData) -> ModelData {
        let text = input.get_text("text");
        let text = vec![text];
        let output = self.model.summarize(&text);
        let output = output.get(0).unwrap().to_owned();
        ModelData::new()
            .with_text("output".to_owned(), output)
    }
}

pub async fn run_simple_text_summarization() {
    let text = r#"some long text here"#;

    let model = TextSummarizationModel::new();
    let input = ModelData::new().with_text("text".to_owned(), text.to_owned());
    let result = model.run(&input);
    let result = result.get_text("output");
    info!("result: {:?}", result);
}