use std::error::Error;

use tauri_build::{AppManifest, Attributes};

fn main() -> Result<(), Box<dyn Error>> {
    let app_manifest = AppManifest::new().commands(&[
        "subscribe_to_open_board",
        "subscribe_to_replica",
        "subscribe_to_channels",
        "subscribe_to_flow",
        "read_flow",
    ]);
    tauri_build::try_build(Attributes::new().app_manifest(app_manifest))?;
    Ok(())
}
