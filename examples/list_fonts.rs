use anyhow::Result;
use regnumassets::{AssetType, ResourceIndex};
use std::fs::File;

fn main() -> Result<()> {
    let f = File::open("examples/regnum/data3.idx")?;
    let index = ResourceIndex::read(f).unwrap();

    let fonts = index.filter_by_type(AssetType::Font);
    for font in &fonts {
        println!(
            "Resource #{}: {}",
            font.resource_id.unwrap_or(0),
            font.name.as_deref().unwrap_or("(unnamed)".into())
        );
    }

    Ok(())
}
