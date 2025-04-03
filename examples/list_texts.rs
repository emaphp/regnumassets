use anyhow::Result;
use regnumassets::{AssetType, ResourceIndex};
use std::fs::File;

fn main() -> Result<()> {
    let f = File::open("examples/regnum/data5.idx")?;
    let index = ResourceIndex::read(f).unwrap();

    let texts = index.filter_by_type(AssetType::Text);

    for text in &texts {
        println!(
            "Resource #{}: {}",
            text.resource_id.unwrap_or(0),
            text.name.as_deref().unwrap_or("(unnamed)".into())
        );
    }

    Ok(())
}
