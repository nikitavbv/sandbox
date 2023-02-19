# running with gpu

- Download pytorch 1.13.0 cuda 11.6 (cxx11 ABI)
- export LIBTORCH and LD_LIBRARY_PATH (see linux-gpu-env.sh)

## Vultr MLDev image

It seems that the key to make things work is to avoid installing any updates. Otherwise, cuda exceptions start to appear.

Stable diffusion model works quite well on 1/7 A100.