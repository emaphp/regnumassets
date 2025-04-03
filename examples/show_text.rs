use anyhow::Result;
use regnumassets::{AssetContent, AssetData, ResourceIndex};
use std::fs::File;

fn main() -> Result<()> {
    let f = File::open("examples/regnum/data5.idx")?;
    let index = ResourceIndex::read(f).unwrap();

    let text = index.get_by_resource_id(59847).unwrap();

    let f = File::open("examples/regnum/data5.sdb")?;
    let asset = AssetData::read(&f, &text).unwrap();

    println!(
        "showing content in {} [{}]",
        asset.asset_name, asset.resource_id
    );

    match asset.content {
        AssetContent::Text { contents } => {
            for content in contents {
                println!("refs: {:?}", content.refs);
                for node in &content.nodes {
                    println!("TEXT: {:?}", node);
                }
            }
        }
        _ => {
            println!("this content is not supported")
        }
    }

    Ok(())
}
