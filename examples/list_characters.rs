use anyhow::Result;
use regnumassets::ResourceIndex;
use std::fs::File;

fn main() -> Result<()> {
    let f = File::open("examples/regnum/live/characters.idx")?;
    let index = ResourceIndex::read(f).unwrap();

    println!("Found characters: {}", index.bookmarks.len());

    for chara in index.bookmarks.iter() {
        println!(
            "Character #{}: {}",
            chara.resource_id.unwrap_or(0),
            chara.name.as_deref().unwrap_or("(unnamed)".into())
        );
    }

    Ok(())
}
