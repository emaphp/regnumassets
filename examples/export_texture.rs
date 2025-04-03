use anyhow::Result;
use regnumassets::{AssetContent, AssetData, ResourceIndex};
use std::fs::File;
use std::io::Write;

fn main() -> Result<()> {
    let f = File::open("examples/regnum/data6.idx")?;
    let index = ResourceIndex::read(f).unwrap();

    let texture = index.get_by_resource_id(85953).unwrap();

    let f = File::open("examples/regnum/data6.sdb")?;
    let asset = AssetData::read(&f, &texture).unwrap();

    match asset.content {
        AssetContent::Texture { width, height, dds } => {
            println!(
                "writing texture '{}' ({}x{}) to out.dds",
                asset.asset_name, width, height
            );

            let mut file = File::create("out.dds")?;
            dds.write(&mut file)?;
            file.flush()?;
        }

        _ => {
            println!("couuld not parse texture asset")
        }
    }

    Ok(())
}
