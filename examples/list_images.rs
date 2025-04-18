use anyhow::Result;
use regnumassets::{AssetType, ResourceIndex};
use std::fs::File;

fn main() -> Result<()> {
    let f = File::open("examples/regnum/data5.idx")?;
    let index = ResourceIndex::read(f).unwrap();

    let images = index.filter_by_type(AssetType::Image);

    for image in &images {
        println!(
            "Resource #{}: {}",
            image.resource_id.unwrap_or(0),
            image.name.as_deref().unwrap_or("(unnamed)".into()),
        );
    }

    Ok(())
}
