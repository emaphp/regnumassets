use anyhow::Result;
use regnumassets::{AssetContent, AssetData, ResourceIndex};
use std::fs::File;
use std::io::Write;

fn main() -> Result<()> {
    let f = File::open("examples/regnum/data2.idx")?;
    let index = ResourceIndex::read(f).unwrap();

    let sound = index.get_by_resource_id(56934).unwrap();

    let f = File::open("examples/regnum/data2.sdb")?;
    let asset = AssetData::read(&f, &sound).unwrap();

    match asset.content {
        AssetContent::Sound {
            filename,
            bytes,
            size,
        } => {
            println!("writing {} bytes file to {}", size, filename);
            let mut output = File::create(filename)?;
            output.write_all(bytes.as_ref())?;
            output.flush()?;
        }
        _ => {
            println!("couuld not parse music asset")
        }
    }

    Ok(())
}
