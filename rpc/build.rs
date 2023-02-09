use std::{io::Result, env, path::PathBuf};

fn main() -> Result<()> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    tonic_build::configure()
        .build_server(is_server_generation_enabled())
        .build_client(is_client_generation_enabled())
        .file_descriptor_set_path(out_dir.join("lab_descriptor.bin"))
        .compile(&["proto/lab.proto"], &["proto"])?;
    Ok(())
}

#[cfg(feature = "server")]
fn is_server_generation_enabled() -> bool {
    true
}

#[cfg(not(feature = "server"))]
fn is_server_generation_enabled() -> bool {
    false
}

#[cfg(feature = "client")]
fn is_client_generation_enabled() -> bool {
    true
}

#[cfg(not(feature = "client"))]
fn is_client_generation_enabled() -> bool {
    false
}
