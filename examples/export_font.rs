use anyhow::Result;
use regnumassets::{asset::Font, AssetContent, AssetData, ResourceIndex};
use std::{fs::File, io::Write};

fn main() -> Result<()> {
    let resource_id: u32 = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "56364".into()) // Cambria Regular
        .parse()
        .expect("resource id must be a number");

    let f = File::open("examples/regnum/data3.idx")?;
    let index = ResourceIndex::read(f).unwrap();

    let bookmark = match index.get_by_resource_id(resource_id) {
        Some(b) if b.asset_type == regnumassets::AssetType::Font => b,
        Some(_) => anyhow::bail!("resource #{} is not a font", resource_id),
        None => anyhow::bail!("resource #{} not found in data3.idx", resource_id),
    };

    let f = File::open("examples/regnum/data3.sdb")?;
    let asset = AssetData::read(&f, &bookmark)?;

    match asset.content {
        AssetContent::Font(Font::TrueType(bytes)) => {
            println!(
                "writing font '{}' ({} bytes) to out.ttf",
                asset.asset_name,
                bytes.len()
            );
            let mut file = File::create("out.ttf")?;
            file.write_all(&bytes)?;
            file.flush()?;
        }
        AssetContent::Font(Font::OpenType(bytes)) => {
            println!(
                "writing font '{}' ({} bytes) to out.otf",
                asset.asset_name,
                bytes.len()
            );
            let mut file = File::create("out.otf")?;
            file.write_all(&bytes)?;
            file.flush()?;
        }
        AssetContent::Font(Font::FontCollection { faces, bytes }) => {
            println!(
                "writing font collection '{}' ({} faces, {} bytes) to out.ttc",
                asset.asset_name,
                faces,
                bytes.len()
            );
            let mut file = File::create("out.ttc")?;
            file.write_all(&bytes)?;
            file.flush()?;
        }
        AssetContent::Font(Font::FontBlob { height, .. }) => {
            println!(
                "asset '{}' is a game bitmap-font blob (height {}) — not a TTF, skipping",
                asset.asset_name, height
            );
        }
        _ => {
            println!("could not parse font asset");
        }
    }

    Ok(())
}
