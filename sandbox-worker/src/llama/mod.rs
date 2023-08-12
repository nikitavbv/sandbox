use {
    candle::{Device, DType, Tensor},
    candle_nn::VarBuilder,
    candle_transformers::generation::LogitsProcessor,
    tokenizers::Tokenizer,
    crate::storage::Storage,
    self::model::{Config, Cache, Llama},
};

mod model;

pub struct LlamaChatModel {
    llama: Llama,
    tokenizer: Tokenizer,
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
            llama,
            tokenizer,
        }
    }

    pub fn chat(&self) {
        let mut tokens = self.tokenizer
            .encode("Hello my dear", true)
            .unwrap()
            .get_ids()
            .to_vec();

        let mut logits_processor = LogitsProcessor::new(42, Some(0.6));
        let mut new_tokens = vec![];
        let device = Device::Cpu;

        let mut index_pos = 0;
        for index in 0..10 {
            let context_size = if index > 0 {
                1
            } else {
                tokens.len()
            };
            let ctxt = &tokens[tokens.len().saturating_sub(context_size)..];
            let input = Tensor::new(ctxt, &device).unwrap().unsqueeze(0).unwrap();
            let logits = self.llama.forward(&input, index_pos).unwrap();
            let logits = logits.squeeze(0).unwrap();
            index_pos += ctxt.len();

            let next_token = logits_processor.sample(&logits).unwrap();
            tokens.push(next_token);
            new_tokens.push(next_token);

            println!("{:?}", self.tokenizer.decode(vec![next_token], true).unwrap());
        }
    }
}