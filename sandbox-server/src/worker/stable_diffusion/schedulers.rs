// copied from: https://github.com/huggingface/candle/blob/main/candle-examples/examples/stable-diffusion/schedulers.rs

//! # Diffusion pipelines and models
//!
//! Noise schedulers can be used to set the trade-off between
//! inference speed and quality.

use candle::{Result, Tensor};

/// This represents how beta ranges from its minimum value to the maximum
/// during training.
#[derive(Debug, Clone, Copy)]
pub enum BetaSchedule {
    /// Linear interpolation.
    Linear,
    /// Linear interpolation of the square root of beta.
    ScaledLinear,
    /// Glide cosine schedule
    SquaredcosCapV2,
}

#[derive(Debug, Clone, Copy)]
pub enum PredictionType {
    Epsilon,
    VPrediction,
    Sample,
}

/// Create a beta schedule that discretizes the given alpha_t_bar function, which defines the cumulative product of
/// `(1-beta)` over time from `t = [0,1]`.
///
/// Contains a function `alpha_bar` that takes an argument `t` and transforms it to the cumulative product of `(1-beta)`
/// up to that part of the diffusion process.
pub(crate) fn betas_for_alpha_bar(num_diffusion_timesteps: usize, max_beta: f64) -> Result<Tensor> {
    let alpha_bar = |time_step: usize| {
        f64::cos((time_step as f64 + 0.008) / 1.008 * std::f64::consts::FRAC_PI_2).powi(2)
    };
    let mut betas = Vec::with_capacity(num_diffusion_timesteps);
    for i in 0..num_diffusion_timesteps {
        let t1 = i / num_diffusion_timesteps;
        let t2 = (i + 1) / num_diffusion_timesteps;
        betas.push((1.0 - alpha_bar(t2) / alpha_bar(t1)).min(max_beta));
    }
    let betas_len = betas.len();
    Tensor::from_vec(betas, betas_len, &candle::Device::Cpu)
}