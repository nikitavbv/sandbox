# running with gpu

- Download pytorch 1.13.0 cuda 11.6 (cxx11 ABI)
- export LIBTORCH and LD_LIBRARY_PATH (see linux-gpu-env.sh)

## GPU Memory usage

stable diffusion image generation model requires 10GB vram (however, there is space for optimizations).

rust-bert default model (gpt2-medium) requires around of 4-5gb of vram.

## Vultr MLDev image

It seems that the key to make things work is to avoid installing any updates. Otherwise, cuda exceptions start to appear.

Stable diffusion model works quite well on 1/7 A100 ($250/month or $0.372/h) - which seems to be ideal for demo purposes.

gpt2-medium works well on 1/10 A100.

## MPS

While MPS is cool, running on CPU is faster on my 2020 M1 Macbook Pro. See https://github.com/pytorch/pytorch/issues/77799 and https://discuss.pytorch.org/t/sequential-throughput-of-gpu-execution/156303 for more details