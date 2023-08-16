use {
    candle::{Device, DType, Tensor},
    candle_nn::VarBuilder,
    candle_transformers::generation::LogitsProcessor,
    tokenizers::Tokenizer,
    super::storage::Storage,
    self::model::{Config, Cache, Llama},
};

mod model;

pub struct LlamaChatModel {
    llama: Llama,
    tokenizer: Tokenizer,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Role {
    User,
    Assistant,
    System,
}

#[derive(Debug)]
pub struct Message {
    role: Role,
    text: String,
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

    pub fn chat(&self, messages: Vec<Message>) -> Message {
        let mut tokens = Vec::new();

        for message in messages.chunks(2) {
            if message[0].role != Role::User {
                panic!("expected role of message to be User");
            }
            if message.len() > 1 && message[1].role != Role::Assistant{
                panic!("expected role of message to be Assistant");
            }

            let prompt = if message.len() == 1 {
                format!("[INST] {} [/INST] ", message[0].text)
            } else {
                format!("[INST] {} [/INST] {} ", message[0].text, message[1].text)
            };

            let mut message_tokens = self.tokenizer
                .encode(prompt, true)
                .unwrap()
                .get_ids()
                .to_vec();

            tokens.append(&mut message_tokens);
        }

        let end_of_sequence = self.tokenizer.token_to_id("</s>").unwrap();

        let mut logits_processor = LogitsProcessor::new(42, Some(0.6));
        let mut new_tokens = vec![];
        let device = Device::Cpu;

        let mut index_pos = 0;
        let max_tokens = 5000;
        let mut index = 0;
        while index < max_tokens {
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
            if next_token == end_of_sequence {
                println!("break because end of sequence");
                break;
            }

            tokens.push(next_token);
            new_tokens.push(next_token);

            println!("{}", self.tokenizer.decode(&new_tokens, false).unwrap());

            index += 1;
        }

        Message::new(Role::Assistant, self.tokenizer.decode(&new_tokens, true).unwrap())
    }
}

impl Message {
    pub fn new(role: Role, message: String) -> Self {
        Self {
            role,
            text: message
        }
    }

    pub fn role(&self) -> &Role {
        &self.role
    }

    pub fn content(&self) -> &str {
        &self.text
    }
}