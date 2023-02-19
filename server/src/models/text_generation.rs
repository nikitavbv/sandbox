use {
    std::time::Instant,
    tracing::info,
    rust_bert::pipelines::text_generation::TextGenerationModel as RustBertTextGenerationModel,
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

    pub fn run(&self) {
        let prompt = "Human: What is your favourite movie?!\n\n AI:";
        let output = self.model.generate(&[prompt], None);
        info!("output is {:?}", output);
    }
}

pub async fn run_simple_text_generation() {
    let model = TextGenerationModel::new();

    let started_at = Instant::now();
    model.run();
    info!("image generated in {} seconds", (Instant::now() - started_at).as_secs());
}