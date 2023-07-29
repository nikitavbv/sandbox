use {
    std::fs,
    tracing::info,
    diffusers::{transformers::clip, pipelines::stable_diffusion},
    tch::{Tensor, Device, nn::Module, Kind},
    tempfile::tempdir,
    rand::Rng,
    crate::storage::Storage,
};

pub struct StableDiffusionImageGenerationModel {
    device: Device,
    sd_config: stable_diffusion::StableDiffusionConfig,
    model: ModelComponents,
}

struct ModelComponents {
    tokenizer: clip::Tokenizer,
    text_model: clip::ClipTextTransformer,
    vae: diffusers::models::vae::AutoEncoderKL,
    unet: diffusers::models::unet_2d::UNet2DConditionModel,
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
        let vocab_file = data_resolver.load_model_file("bpe_simple_vocab_16e6.txt").await;
        let clip_weights = data_resolver.load_model_file("clip_v2.1.ot").await;
        let vae_weights = data_resolver.load_model_file("vae.ot").await;
        let unet_weights = data_resolver.load_model_file("unet.ot").await;

        let tokenizer = clip::Tokenizer::create(&vocab_file, &sd_config.clip).unwrap();
        let text_model = sd_config.build_clip_transformer(&clip_weights, device).unwrap();
        let vae = sd_config.build_vae(&vae_weights, device).unwrap();
        let unet = sd_config.build_unet(&unet_weights, device, 4).unwrap();

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
        let device = Device::cuda_if_available();

        let mut sd_config = stable_diffusion::StableDiffusionConfig::v2_1(None, Some(512), Some(512));

        let model = ModelComponents::new(storage, &mut sd_config, device).await;

        Self {
            device,
            sd_config,
            model,
        }
    }

    pub fn run(&self, prompt: &str, updates_callback: tokio::sync::mpsc::UnboundedSender<ImageGenerationStatus>) -> Vec<u8> {
        info!("using device: {:?}", self.device);

        let uncond_prompt = "";
        let seed: i64 = rand::thread_rng().gen();
        let n_steps = 30;
        let guidance_scale = 7.5;

        let scheduler = self.sd_config.build_scheduler(n_steps);

        let tokens = self.model.tokenizer.encode(&prompt).unwrap();
        let tokens: Vec<i64> = tokens.into_iter().map(|x| x as i64).collect();
        let tokens = Tensor::from_slice(&tokens).view((1, -1)).to(self.device);
        let uncond_tokens = self.model.tokenizer.encode(uncond_prompt).unwrap();
        let uncond_tokens: Vec<i64> = uncond_tokens.into_iter().map(|x| x as i64).collect();
        let uncond_tokens = Tensor::from_slice(&uncond_tokens).view((1, -1)).to(self.device);

        let _no_grad_guard = tch::no_grad_guard();

        let text_embeddings = self.model.text_model.forward(&tokens);
        let uncond_embeddings = self.model.text_model.forward(&uncond_tokens);
        let text_embeddings = Tensor::cat(&[uncond_embeddings, text_embeddings], 0).to(self.device);
    
        let output_dir = tempdir().unwrap(); 

        let bsize = 1;
        tch::manual_seed(seed + 0);
        let mut latents = Tensor::randn(
            &[bsize, 4, self.sd_config.height / 8, self.sd_config.width / 8],
            (Kind::Float, self.device)
        );

        latents *= scheduler.init_noise_sigma();

        for (timestep_index, &timestep) in scheduler.timesteps().iter().enumerate() {
            info!("running timestep index: {}", timestep_index);
            updates_callback.send(ImageGenerationStatus::InProgress { current_step: timestep_index as u32, total_steps: n_steps as u32 }).unwrap();

            let latent_model_input = Tensor::cat(&[&latents, &latents], 0);

            let latent_model_input = scheduler.scale_model_input(latent_model_input, timestep);
            let noise_pred = self.model.unet.forward(&latent_model_input, timestep as f64, &text_embeddings);
            let noise_pred = noise_pred.chunk(2, 0);
            let (noise_pred_uncond, noise_pred_text) = (&noise_pred[0], &noise_pred[1]);
            let noise_pred =
                noise_pred_uncond + (noise_pred_text - noise_pred_uncond) * guidance_scale;
            latents = scheduler.step(&noise_pred, timestep, &latents);
        }

        let latents = latents.to(self.device);
        let image = self.model.vae.decode(&(&latents / 0.18215));
        let image = (image / 2 + 0.5).clamp(0.0, 1.0).to_device(Device::Cpu);
        let image = (image * 255.0).to_kind(Kind::Uint8);

        let output_file = output_dir.path().join("output.png");
        tch::vision::image::save(&image, &output_file).unwrap();

        let result = fs::read(output_file).unwrap();

        result
    }
}
