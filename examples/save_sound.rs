use anyhow::Result;
use regnumassets::{AssetData, ResourceIndex};
use std::fs::File;
use std::io::Write;

fn main() -> Result<()> {
    let f = File::open("data2.idx")?;
    let index = ResourceIndex::new(f).unwrap();

    let sound = index.get_by_resource_id(50677).unwrap();

    let f = File::open("data2.sdb")?;
    let asset = AssetData::new(&f, &sound).unwrap();

    let filename = asset.filename.unwrap();
    println!("Writing file to {}", filename);

    let mut output = File::create(filename)?;
    output.write_all(asset.bytes.unwrap().as_ref())?;
    output.flush()?;

    Ok(())
}
