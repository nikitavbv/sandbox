// most of the implementation is taken from candle examples: https://github.com/huggingface/candle/tree/main/candle-examples/examples/stable-diffusion

use {
    std::fs,
    tracing::info,
    tempfile::tempdir,
    candle::{DType, Device, Tensor, IndexOp},
    tokenizers::Tokenizer,
    super::storage::Storage,
    self::{
        vae::AutoEncoderKL,
        unet_2d::UNet2DConditionModel,
    },
};

pub mod attention;
pub mod clip;
pub mod ddim;
pub mod embeddings;
pub mod resnet;
pub mod schedulers;
pub mod stable_diffusion;
pub mod unet_2d;
pub mod unet_2d_blocks;
pub mod utils;
pub mod vae;

pub struct StableDiffusionImageGenerationModel {
    device: Device,
    sd_config: stable_diffusion::StableDiffusionConfig,
    model: ModelComponents,
}

struct ModelComponents {
    tokenizer: Tokenizer,
    text_model: clip::ClipTextTransformer,
    vae: AutoEncoderKL,
    unet: UNet2DConditionModel,
}

#[derive(Debug)]
pub enum ImageGenerationStatus {
    StartedImageGeneration {
        current_image: u32,
    },
    InProgress {
        current_step: u32,
        total_steps: u32,
    },
    Finished,
}

impl ModelComponents {
    async fn new(data_resolver: &Storage, sd_config: &mut stable_diffusion::StableDiffusionConfig, device: Device) -> Self {
        let tokenizer_file = data_resolver.load_model_file("stable-diffusion", "tokenizer.json").await;
        let clip_weights = data_resolver.load_model_file("stable-diffusion", "text-encoder-model.safetensors").await;
        let vae_weights = data_resolver.load_model_file("stable-diffusion", "vae-model.safetensors").await;
        let unet_weights = data_resolver.load_model_file("stable-diffusion", "unet-model.safetensors").await;

        let tokenizer = Tokenizer::from_file(tokenizer_file).unwrap();
        let text_model = sd_config.build_clip_transformer(&clip_weights, &device).unwrap();
        let vae = sd_config.build_vae(&vae_weights, &device).unwrap();
        let unet = sd_config.build_unet(&unet_weights, &device, 4).unwrap();

        Self {
            tokenizer,
            text_model,
            vae,
            unet,
        }
    }
}

impl StableDiffusionImageGenerationModel {
    pub async fn new(storage: &Storage) -> Self {
        Self::init(storage).await
    }

    async fn init(storage: &Storage) -> Self {
        let device = Device::cuda_if_available(0).unwrap();

        let mut sd_config = stable_diffusion::StableDiffusionConfig::v2_1(None, Some(512), Some(512));

        let model = ModelComponents::new(storage, &mut sd_config, device.clone()).await;

        Self {
            device,
            sd_config,
            model,
        }
    }

    pub fn run(&self, prompt: &str, updates_callback: tokio::sync::mpsc::UnboundedSender<ImageGenerationStatus>) -> Vec<u8> {
        info!("using device: {:?}", self.device);

        let n_steps = 30;
        let guidance_scale = 7.5;

        let scheduler = self.sd_config.build_scheduler(n_steps).unwrap();

        let pad_id = match &self.sd_config.clip.pad_with {
            Some(padding) => *self.model.tokenizer.get_vocab(true).get(padding.as_str()).unwrap(),
            None => *self.model.tokenizer.get_vocab(true).get("<|endoftext|>").unwrap(),
        };

        let mut tokens = self.model.tokenizer.encode(prompt, true).unwrap().get_ids().to_vec();
        while tokens.len() < self.sd_config.clip.max_position_embeddings {
            tokens.push(pad_id);
        }
        let tokens = Tensor::new(tokens.as_slice(), &self.device).unwrap().unsqueeze(0).unwrap();

        let mut uncond_tokens = self.model.tokenizer.encode("", true).unwrap().get_ids().to_vec();
        while uncond_tokens.len() < self.sd_config.clip.max_position_embeddings {
            uncond_tokens.push(pad_id);
        }
        let uncond_tokens = Tensor::new(uncond_tokens.as_slice(), &self.device).unwrap().unsqueeze(0).unwrap();

        let text_embeddings = self.model.text_model.forward(&tokens).unwrap();
        let uncond_embeddings = self.model.text_model.forward(&uncond_tokens).unwrap();
        let text_embeddings = Tensor::cat(&[uncond_embeddings, text_embeddings], 0).unwrap();
    
        let output_dir = tempdir().unwrap(); 

        let bsize = 1;
        let mut latents = Tensor::randn(
            0f32,
            1f32,
            (bsize, 4, self.sd_config.height / 8, self.sd_config.width / 8),
            &self.device,
        ).unwrap();

        latents = (latents * scheduler.init_noise_sigma()).unwrap();

        for (timestep_index, &timestep) in scheduler.timesteps().iter().enumerate() {
            info!("running timestep index: {}", timestep_index);
            updates_callback.send(ImageGenerationStatus::InProgress { current_step: timestep_index as u32, total_steps: n_steps as u32 }).unwrap();

            let latent_model_input = Tensor::cat(&[&latents, &latents], 0).unwrap();

            let latent_model_input = scheduler.scale_model_input(latent_model_input, timestep).unwrap();
            let noise_pred = self.model.unet.forward(&latent_model_input, timestep as f64, &text_embeddings).unwrap();
            let noise_pred = noise_pred.chunk(2, 0).unwrap();
            let (noise_pred_uncond, noise_pred_text) = (&noise_pred[0], &noise_pred[1]);
            let noise_pred =
                (noise_pred_uncond + (noise_pred_text - noise_pred_uncond).unwrap() * guidance_scale).unwrap();
            latents = scheduler.step(&noise_pred, timestep, &latents).unwrap();
        }

        updates_callback.send(ImageGenerationStatus::InProgress { current_step: (n_steps - 1) as u32, total_steps: n_steps as u32 }).unwrap();

        let image = self.model.vae.decode(&(&latents / 0.18215).unwrap()).unwrap();
        let image = ((image / 2.).unwrap() + 0.5).unwrap().to_device(&Device::Cpu).unwrap();
        let image = (image * 255.0).unwrap().to_dtype(DType::U8).unwrap().i(0).unwrap();

        let output_file = output_dir.path().join("./output.png");
        utils::save_image(&image, &output_file).unwrap();
        fs::read(output_file).unwrap()
    }
}
