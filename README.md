<div align="center">

<a href="https://sandbox.nikitavbv.com"><img src="./docs/preview1.png" width="400"></a>
<a href="https://sandbox.nikitavbv.com/tasks/wkrnakbAKeP1re"><img src="./docs/preview2.png" width="400"></a>

sandbox: web app for exploring generative ai models
</div>

---

This web app is built for learning and fun purposes. All components are written in Rust.

# Usage

- publicly available instance at [sandbox.nikitavbv.com](https://sandbox.nikitavbv.com)
- self hosted

# Features

- Generate images with Stable Diffusion v2.1

# TODOs

- generate images using controlnet.
- chat with llama.
- serve static frontend files from sandbox-server, so that you can run most of the app (without worker) with single `cargo run`.
- simple self hosting.
- enable caching for assets.
- make "tasks" link in the header to be an actual link.
- delete images and tasks.
- button to generate X more images for task.
- bidirectional streaming between worker and server.
- add some metrics to control how many tasks are completed/running/in queue and how much time it takes to process a task.
- worker k8s config

# Acknowledgments

Most of the heavy lifting is performed by [candle](https://github.com/huggingface/candle) (which is an amazing library) and code samples from candle examples.
