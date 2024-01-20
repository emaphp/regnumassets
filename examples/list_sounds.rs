use anyhow::Result;
use regnumassets::{AssetType, ResourceIndex};
use std::fs::File;

fn main() -> Result<()> {
    let f = File::open("data2.idx")?;
    let index = ResourceIndex::new(f).unwrap();

    let sounds = index.filter_by_type(AssetType::Sound);

    for sound in &sounds {
        println!(
            "Resource #{}: {}",
            sound.resource_id,
            sound.name.as_deref().unwrap_or("(unnamed)".into())
        );
    }

    Ok(())
}
