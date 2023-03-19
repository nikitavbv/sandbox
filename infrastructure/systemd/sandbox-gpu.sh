#!/bin/bash

export HOME=/root

if [ ! -d "/root/.cargo" ]
then
    echo "Rust is not installed. Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
else
    echo "Rust is already installed."
fi

if [ ! -d "/root/sandbox" ]
then
    echo "Directory /root/sandbox does not exist. Cloning repository..."
    git clone https://github.com/nikitavbv/sandbox.git /root/sandbox
else
    echo "Directory /root/sandbox already exists."
fi

cd /root/sandbox

source /root/.cargo/env
source ./linux-gpu-env.sh

cargo run --release