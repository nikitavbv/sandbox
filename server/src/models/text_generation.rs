use {
    std::time::Instant,
    tracing::info,
    rust_bert::pipelines::text_generation::TextGenerationModel as RustBertTextGenerationModel,
    crate::models::io::ModelInput,
};

pub struct TextGenerationModel {
    model: RustBertTextGenerationModel,
}

impl TextGenerationModel {
    pub fn new() -> Self {
        Self {
            model: RustBertTextGenerationModel::new(Default::default()).unwrap(),
        }
    }

    pub fn run(&self, input: &ModelInput) {
        let prompt = input.get_text("prompt");
        let output = self.model.generate(&[prompt], None);
        info!("output is {:?}", output);
    }
}

pub async fn run_simple_text_generation() {
    let model = TextGenerationModel::new();

    let started_at = Instant::now();
    let input = ModelInput::new().with_text("prompt".to_owned(), "Human: What is your favourite movie?\n\nAI:".to_owned());
    model.run(&input);
    info!("image generated in {} seconds", (Instant::now() - started_at).as_secs());
}