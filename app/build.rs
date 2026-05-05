use std::path::PathBuf;
use wgsl_bindgen::{NalgebraWgslTypeMap, WgslBindgenOptionBuilder, WgslTypeSerializeStrategy};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let shader_dir: String = std::env::var("SHADERS").expect("Unable to get $env:SHADERS.");

    let shader_path = if shader_dir.starts_with("~/") {
        let home = std::env::var("HOME").expect("Unable to get HOME directory");
        PathBuf::from(shader_dir.replacen("~/", &format!("{}/", home), 1))
    } else {
        PathBuf::from(&shader_dir)
    };

    let entries = std::fs::read_dir(&shader_path)
        .expect("Unable to access the shaders directory.")
        .filter_map(|res| res.ok())
        .filter(|entry| {
            entry.path().is_file() && entry.path().extension().map_or(false, |ext| ext == "wgsl")
        })
        .collect::<Vec<_>>();

    let mut builder = WgslBindgenOptionBuilder::default();
    let option_builder = builder.workspace_root(shader_path.to_str().unwrap());

    for entry in entries {
        let path = entry.path();
        option_builder.add_entry_point(path.to_string_lossy().to_string());
    }

    option_builder
        .serialization_strategy(WgslTypeSerializeStrategy::Bytemuck)
        .type_map(NalgebraWgslTypeMap)
        .output("app/src/bindings.rs")
        .build()?
        .generate()?;

    Ok(())
}
