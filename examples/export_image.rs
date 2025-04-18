use anyhow::Result;
use regnumassets::{AssetContent, AssetData, ResourceIndex};
use std::fs::File;
use std::io::Write;

fn main() -> Result<()> {
    let f = File::open("examples/regnum/data5.idx")?;
    let index = ResourceIndex::read(f).unwrap();

    let image = index.get_by_resource_id(75879).unwrap();

    let f = File::open("examples/regnum/data5.sdb")?;
    let asset = AssetData::read(&f, &image).unwrap();

    match asset.content {
        AssetContent::Image { bytes } => {
            let filename = "out.jpeg";
            println!("writing file to {}", filename);
            let mut output = File::create(filename)?;
            output.write_all(bytes.as_ref())?;
            output.flush()?;
        }
        _ => {
            println!("couuld not parse image asset")
        }
    }

    Ok(())
}
