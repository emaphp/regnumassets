use anyhow::Result;
use regnumassets::{AssetType, ResourceIndex};
use std::fs::File;

fn main() -> Result<()> {
    let f = File::open("examples/regnum/data6.idx")?;
    let index = ResourceIndex::read(f).unwrap();

    let textures = index.filter_by_type(AssetType::Texture);

    for texture in &textures {
        println!(
            "Resource #{}: {}",
            texture.resource_id.unwrap_or(0),
            texture.name.as_deref().unwrap_or("(unnamed)".into()),
        );
    }

    Ok(())
}
