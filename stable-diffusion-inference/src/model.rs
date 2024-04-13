use tracing::info;

// based on https://github.com/huggingface/candle/blob/main/candle-examples/examples/stable-diffusion/main.rs
pub struct ImageGenerationModel {
}

impl ImageGenerationModel {
    pub fn load() -> Self {
        info!("loading model");

        Self {
        }
    }

    pub fn generate_image(&mut self, prompt: &str) -> Vec<u8> {
        let timesteps = scheduler.timesteps();
        let latents = (Tensor::randn(
            0f32,
            1f32,
            (bsize, 4, height / 8, width / 8),
            &device,
        ).unwrap() * scheduler.init_noise_sigma()).unwrap();
        let mut latents = latents.to_dtype(dtype).unwrap();

        for (timestep_index, &timestep) in timesteps.iter().enumerate() {
            let latent_model_input = latents.clone();

            let latent_model_input = scheduler.scale_model_input(latent_model_input, timestep).unwrap();
            let noise_pred = unet.forward(&latent_model_input, timestep as f64, &text_embeddings).unwrap();

            latents = scheduler.step(&noise_pred, timestep, &latents).unwrap();
        }

        let image = vae.decode(&(&latents / vae_scale).unwrap()).unwrap();
        let image = ((image / 2.).unwrap() + 0.5).unwrap().to_device(&Device::Cpu).unwrap();
        let image = (image.clamp(0f32, 1.).unwrap() * 255.0).unwrap().to_dtype(DType::U8).unwrap().i(0).unwrap();
        // TODO: save image

        unimplemented!()
    }
}
