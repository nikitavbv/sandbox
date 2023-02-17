use {
    std::time::Instant,
    tracing::info,
    diffusers::{transformers::clip, pipelines::stable_diffusion},
    tch::{Tensor, Device, nn::Module, Kind},
    config::Config,
    crate::data::{
        file::FileDataResolver,
        object_storage::ObjectStorageDataResolver,
        cached_resolver::CachedResolver,
        resolver::DataResolver,
    },
};

pub struct StableDiffusionImageGenerationModel {
    device: Device,
    sd_config: stable_diffusion::StableDiffusionConfig,
    tokenizer: clip::Tokenizer,
    text_model: clip::ClipTextTransformer,
    vae: diffusers::models::vae::AutoEncoderKL,
    unet: diffusers::models::unet_2d::UNet2DConditionModel,
}

impl StableDiffusionImageGenerationModel {
    pub async fn new<T: DataResolver>(data_resolver: T) -> Self {
        Self::init(data_resolver).await
    }

    async fn init<T: DataResolver>(data_resolver: T) -> Self {
        let vocab_file = data_resolver.resolve_to_fs_path("bpe_simple_vocab_16e6.txt").await.unwrap();
        let clip_weights = data_resolver.resolve_to_fs_path("clip_v2.1.ot").await.unwrap();
        let vae_weights = data_resolver.resolve_to_fs_path("vae.ot").await.unwrap();
        let unet_weights = data_resolver.resolve_to_fs_path("unet.ot").await.unwrap();

        let device = Device::cuda_if_available();

        let sd_config = stable_diffusion::StableDiffusionConfig::v2_1(None);
        let tokenizer = clip::Tokenizer::create(&vocab_file, &sd_config.clip).unwrap();
        let text_model = sd_config.build_clip_transformer(&clip_weights, device).unwrap();
        let vae = sd_config.build_vae(&vae_weights, device).unwrap();
        let unet = sd_config.build_unet(&unet_weights, device, 4).unwrap();

        Self {
            device,
            sd_config,
            tokenizer,
            text_model,
            vae,
            unet,
        }
    }

    pub fn run(&self) {
        info!("using device: {:?}", self.device);

        let prompt = "Orange cat looking into window";
        let uncond_prompt = "";
        let num_samples = 1;
        let seed = 32;
        let n_steps = 30;
        let guidance_scale = 7.5;

        let scheduler = self.sd_config.build_scheduler(n_steps);

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
    
        let bsize = 1;
        for idx in 0..num_samples {
            tch::manual_seed(seed + idx);
            let mut latents = Tensor::randn(
                &[bsize, 4, self.sd_config.height / 8, self.sd_config.width / 8],
                (Kind::Float, self.device)
            );

            latents *= scheduler.init_noise_sigma();

            for (timestep_index, &timestep) in scheduler.timesteps().iter().enumerate() {
                info!("running timestep index: {}", timestep_index);

                let latent_model_input = Tensor::cat(&[&latents, &latents], 0);

                let latent_model_input = scheduler.scale_model_input(latent_model_input, timestep);
                let noise_pred = self.unet.forward(&latent_model_input, timestep as f64, &text_embeddings);
                let noise_pred = noise_pred.chunk(2, 0);
                let (noise_pred_uncond, noise_pred_text) = (&noise_pred[0], &noise_pred[1]);
                let noise_pred =
                    noise_pred_uncond + (noise_pred_text - noise_pred_uncond) * guidance_scale;
                latents = scheduler.step(&noise_pred, timestep, &latents);
            }

            let latents = latents.to(self.device);
            let image = self.vae.decode(&(&latents / 0.18215));
            let image = (image / 2 + 0.5).clamp(0.0, 1.0).to_device(Device::Cpu);
            let image = (image * 255.0).to_kind(Kind::Uint8);
            tch::vision::image::save(&image, format!("./output-{}.png", idx)).unwrap();
        }
    }
}

pub async fn run_simple_image_generation(config: &Config) {
    tch::maybe_init_cuda();

    let object_storage_resolver = ObjectStorageDataResolver::new_with_config(
        "nikitavbv-sandbox".to_owned(), 
        "data/models/stable-diffusion".to_owned(), 
        config
    );

    let file_resolver = FileDataResolver::new("./data/stable-diffusion".to_owned());
    let data_resolver = CachedResolver::new(object_storage_resolver, file_resolver);

    let model = StableDiffusionImageGenerationModel::new(data_resolver).await;
    
    let started_at = Instant::now();
    model.run();
    info!("image generated in {} seconds", (Instant::now() - started_at).as_secs());
}