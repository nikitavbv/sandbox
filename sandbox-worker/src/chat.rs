use {
    std::path::Path,
    llm::{TokenizerSource, models::Llama},
    crate::storage::Storage,
};

pub struct LlamaChatModel {
    llama: Llama,
}

impl LlamaChatModel {
    pub async fn new(storage: &Storage) -> Self {
        let llama = llm::load::<llm::models::Llama>(
            Path::new(&storage.load_model_file("llama", "llama-7b.ggmlv3.q8_0.bin").await),
            TokenizerSource::Embedded,
            Default::default(),
            llm::load_progress_callback_stdout
        ).unwrap();

        Self {
            llama,
        }
    }
}