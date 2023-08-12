use {
    candle::{Device, DType},
    candle_nn::VarBuilder,
    tokenizers::Tokenizer,
    crate::storage::Storage,
    self::model::{Config, Cache, Llama},
};

mod model;

pub struct LlamaChatModel {
    //llama: Llama,
}

impl LlamaChatModel {
    pub async fn new(storage: &Storage) -> Self {
        let device = Device::Cpu;
        let config = Config::config_7b_v2();
        let cache = Cache::new(true, DType::F32, &config, &device).unwrap();

        let handles: Vec<_> = vec![
            storage.load_model_file("llama", "model-00001-of-00002.safetensors").await,
            storage.load_model_file("llama", "model-00002-of-00002.safetensors").await,
        ].iter()
            .map(|f| unsafe { candle::safetensors::MmapedFile::new(f).unwrap() })
            .collect();

        let tensors: Vec<_> = handles
            .iter()
            .map(|h| h.deserialize().unwrap())
            .collect();

        let vb = VarBuilder::from_safetensors(tensors, DType::F32, &device);
        let llama = Llama::load(vb, &cache, &config).unwrap();

        let tokenizer = storage.load_model_file("llama", "tokenizer.json").await;
        let tokenizer = Tokenizer::from_file(tokenizer).unwrap();

        Self {
            // llama,
        }
    }
}