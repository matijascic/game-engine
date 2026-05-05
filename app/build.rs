use wgsl_bindgen::{NalgebraWgslTypeMap, WgslBindgenOptionBuilder, WgslTypeSerializeStrategy};

fn main() -> Result<(), String> {
    let shader_dir: String = std::env::var("SHADERS").expect("Unable to get $env:SHADERS.");

    let entries = std::fs::read_dir(&shader_dir)
        .expect("Unable to access the shaders directory.")
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()
        .unwrap();

    let mut builder = WgslBindgenOptionBuilder::default();

    let option_builder = builder.workspace_root(shader_dir);
    entries.iter().for_each(|p| {
        option_builder.add_entry_point(p.display().to_string());
    });
    option_builder
        .serialization_strategy(WgslTypeSerializeStrategy::Bytemuck)
        .type_map(NalgebraWgslTypeMap)
        .output("src/bindings.rs")
        .build()
        .unwrap()
        .generate()
        .unwrap();

    Ok(())
}
