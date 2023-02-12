use {
    tracing::info,
    diffusers::{transformers::clip, pipelines::stable_diffusion},
    tch::{Tensor, Device},
};

pub struct StableDiffusionImageGenerationModel {
}

impl StableDiffusionImageGenerationModel {
    pub fn new() -> Self {
        Self {
        }
    }
}

pub fn run_simple_image_generation() {
    let prompt = "Orange cat looking into window";
    let uncond_prompt = "";

    tch::maybe_init_cuda();
    
    let model = StableDiffusionImageGenerationModel::new();
    
    let vocab_file = "./server/data/stable-diffusion/bpe_simple_vocab_16e6.txt";
    let clip_weights = "./server/data/stable-diffusion/clip_v2.1.ot";
    let sliced_attention_size = None;

    let device = Device::cuda_if_available();

    let sd_config = stable_diffusion::StableDiffusionConfig::v2_1(sliced_attention_size);
    let tokenizer = clip::Tokenizer::create(vocab_file, &sd_config.clip).unwrap();
    let tokens = tokenizer.encode(&prompt).unwrap();
    let tokens: Vec<i64> = tokens.into_iter().map(|x| x as i64).collect();
    let tokens = Tensor::of_slice(&tokens).view((1, -1)).to(device);
    let uncond_tokens = tokenizer.encode(uncond_prompt).unwrap();
    let uncond_tokens: Vec<i64> = uncond_tokens.into_iter().map(|x| x as i64).collect();
    let uncond_tokens = Tensor::of_slice(&uncond_tokens).view((1, -1)).to(device);

    let no_grad_guard = tch::no_grad_guard();

    // build the clip transformer
    let text_model = sd_config.build_clip_transformer(&clip_weights, device).unwrap();
    // TODO: continue this
}