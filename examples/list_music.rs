use anyhow::Result;
use regnumassets::{AssetType, ResourceIndex};
use std::fs::File;

fn main() -> Result<()> {
    let f = File::open("examples/regnum/data2.idx")?;
    let index = ResourceIndex::read(f).unwrap();

    let sounds = index.filter_by_type(AssetType::Music);

    for sound in &sounds {
        println!(
            "Resource #{}: {}",
            sound.resource_id.unwrap_or(0),
            sound.name.as_deref().unwrap_or("(unnamed)".into())
        );
    }

    Ok(())
}
