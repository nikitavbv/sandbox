use {
    tracing::info,
    diffusers::{transformers::clip, pipelines::stable_diffusion},
    tch::{Tensor, Device, nn::Module},
};

pub struct StableDiffusionImageGenerationModel {
    vocab_file: String,
    clip_weights: String,
    vae_weights: String,
    unet_weights: String,

    device: Device,
    sd_config: stable_diffusion::StableDiffusionConfig,
    tokenizer: clip::Tokenizer,
    text_model: clip::ClipTextTransformer,
}

impl StableDiffusionImageGenerationModel {
    pub fn new() -> Self {
        let data_path = "./server/data/stable-diffusion/";
        let vocab_file = format!("{}{}", data_path, "bpe_simple_vocab_16e6.txt");
        let clip_weights = format!("{}{}", data_path, "clip_v2.1.ot");

        let device = Device::cuda_if_available();

        let sd_config = stable_diffusion::StableDiffusionConfig::v2_1(None);
        let tokenizer = clip::Tokenizer::create(&vocab_file, &sd_config.clip).unwrap();
        let text_model = sd_config.build_clip_transformer(&clip_weights, device).unwrap();

        Self {
            vocab_file,
            clip_weights,
            vae_weights: format!("{}{}", data_path, "vae.ot"),
            unet_weights: format!("{}{}", data_path, "unet.ot"),

            device,
            sd_config,
            tokenizer,
            text_model,
        }
    }

    pub fn run(&self) {
        let prompt = "Orange cat looking into window";
        let uncond_prompt = "";

        let tokens = self.tokenizer.encode(&prompt).unwrap();
        let tokens: Vec<i64> = tokens.into_iter().map(|x| x as i64).collect();
        let tokens = Tensor::of_slice(&tokens).view((1, -1)).to(self.device);
        let uncond_tokens = self.tokenizer.encode(uncond_prompt).unwrap();
        let uncond_tokens: Vec<i64> = uncond_tokens.into_iter().map(|x| x as i64).collect();
        let uncond_tokens = Tensor::of_slice(&uncond_tokens).view((1, -1)).to(self.device);

        let no_grad_guard = tch::no_grad_guard();

        let text_embeddings = self.text_model.forward(&tokens);
        let uncond_embeddings = self.text_model.forward(&uncond_tokens);
        let text_embeddings = Tensor::cat(&[uncond_embeddings, text_embeddings], 0).to(self.device);

        let vae = &self.sd_config.build_vae(&self.vae_weights, self.device).unwrap();

        let unet = &self.sd_config.build_unet(&self.unet_weights, self.device, 4).unwrap();
    }
}

pub fn run_simple_image_generation() {
    tch::maybe_init_cuda();

    let model = StableDiffusionImageGenerationModel::new();
    model.run();
}