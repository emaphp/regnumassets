use super::attribute::{read_attribute, AssetAttribute};
use super::bookmark::AssetBookmark;
use super::{AssetType, ASSET_NODE_START};
use crate::errors::AssetErrors;
use anyhow::{anyhow, Result};
use std::io::{Read, Seek, SeekFrom};

/// A struct containing the data retrieved from an asset database file
#[derive(Default, Clone)]
pub struct AssetData {
    pub asset_type: Option<AssetType>,
    pub resource_id: u32,
    pub filename: Option<String>,
    pub bytes: Option<Vec<u8>>,
    pub size: Option<u32>,
    pub animation: Option<String>,
    pub mesh: Option<String>,
    pub texture: Option<String>,
    pub material: Option<String>,
    pub music: Option<String>,
    pub sound: Option<String>,
}

impl AssetData {
    pub fn new<T: Read + Seek>(mut reader: T, bookmark: &AssetBookmark) -> Result<Self> {
        let pos = bookmark.node_end;
        reader.seek(SeekFrom::Start(pos as u64))?;

        let mut buffer = vec![0; 4];
        reader.read(&mut buffer)?;
        let node = String::from_utf8(buffer).unwrap();
        assert_eq!(node, ASSET_NODE_START);

        // TODO: ???
        let mut buffer = [0; 4];
        reader.read(&mut buffer)?;

        // uid length
        let mut buffer = [0; 1];
        reader.read(&mut buffer)?;
        let [length] = buffer[..] else {
            return Err(anyhow!(AssetErrors::ParserError).context("asset data uid length"));
        };

        // TODO: ??
        let mut buffer = [0; 16];
        reader.read(&mut buffer)?;

        // uid
        let mut buffer = vec![0; length as usize];
        reader.read(&mut buffer)?;

        // name length
        let mut buffer = [0; 1];
        reader.read(&mut buffer)?;
        let [length] = buffer[..] else {
            return Err(anyhow!(AssetErrors::ParserError).context("asset data name length"));
        };

        // name
        let mut buffer = vec![0; length as usize];
        reader.read(&mut buffer)?;

        // TODO: ???
        let mut buffer = [0; 4];
        reader.read(&mut buffer)?;
        assert_eq!(buffer, [0, 0, 0, 0]);

        // size
        let mut buffer = [0; 4];
        reader.read(&mut buffer)?;
        let size = unsafe {
            let (_, values, _) = buffer.align_to::<u32>();
            values[0]
        };
        assert_eq!(size, bookmark.size);

        // some attributes contain binary data that has yet to be reversed
        // these are generally at the end
        // we can bypass them by moving to the end position,
        // which should contain the "RIAP" string
        let the_end = reader.stream_position().unwrap() + size as u64 + 16u64;

        // TODO: ???
        let mut buffer = [0; 16];
        reader.read(&mut buffer)?;

        let mut asset = AssetData::default();
        let limit = bookmark.node_next as u64 - 4;

        loop {
            // fail-safe exit
            let pos = reader.stream_position()?;
            if pos >= limit {
                break;
            }

            let attr = read_attribute(&mut reader)?;

            match attr {
                AssetAttribute::Blank => {}
                AssetAttribute::Start {
                    resource_id,
                    unknown: _,
                } => {
                    asset.resource_id = resource_id;
                }
                AssetAttribute::End => {
                    break;
                }
                AssetAttribute::Animation(animation) => {
                    asset.animation = Some(animation);
                    reader.seek(SeekFrom::Start(the_end))?;
                }
                AssetAttribute::Mesh(mesh) => {
                    asset.mesh = Some(mesh);
                    reader.seek(SeekFrom::Start(the_end))?;
                }
                AssetAttribute::Material(material) => {
                    asset.material = Some(material);
                    reader.seek(SeekFrom::Start(the_end))?;
                }
                AssetAttribute::Texture(texture) => {
                    asset.texture = Some(texture);
                    reader.seek(SeekFrom::Start(the_end))?;
                }
                AssetAttribute::Sound(sound) => {
                    asset.sound = Some(sound);
                }
                AssetAttribute::Music(music) => {
                    asset.music = Some(music);
                }
                AssetAttribute::Filename(filename) => {
                    asset.filename = Some(filename);
                }
                AssetAttribute::Content { size, bytes } => {
                    asset.size = Some(size);
                    asset.bytes = Some(bytes);
                }
            }
        }

        asset.asset_type = {
            if let Some(_) = &asset.texture {
                Some(AssetType::Texture)
            } else if let Some(_) = &asset.animation {
                Some(AssetType::Animation)
            } else if let Some(_) = &asset.mesh {
                Some(AssetType::Mesh)
            } else if let Some(_) = &asset.material {
                Some(AssetType::Material)
            } else if let Some(_) = &asset.sound {
                Some(AssetType::Sound)
            } else if let Some(_) = &asset.music {
                Some(AssetType::Music)
            } else {
                None
            }
        };

        Ok(asset)
    }
}

#[cfg(test)]
mod tests {
    use crate::{AssetData, AssetType, ResourceIndex};
    use std::fs::File;

    #[test]
    fn test_music_database() {
        let f = File::open("data2.idx");
        assert!(f.is_ok());

        let index = ResourceIndex::new(f.unwrap()).unwrap();
        let music = index.bookmarks.get(2).unwrap();
        let sound = index.bookmarks.get(4).unwrap();

        let f = File::open("data2.sdb");
        assert!(f.is_ok());
        let f = f.unwrap();

        let asset = AssetData::new(&f, &music).unwrap();
        assert_eq!(asset.asset_type, Some(AssetType::Music));
        assert_eq!(asset.resource_id, 50203);
        assert_eq!(asset.size, Some(1697043));
        assert_eq!(asset.filename, Some("regnum_combat2.ogg".into()));
        assert_eq!(asset.music, Some("regnum_combat_2".into()));
        assert_eq!(asset.sound, None);

        let asset = AssetData::new(&f, &sound).unwrap();
        assert_eq!(asset.asset_type, Some(AssetType::Sound));
        assert_eq!(asset.resource_id, 50677);
        assert_eq!(asset.size, Some(12703));
        assert_eq!(asset.filename, Some("combat_pain_male_3.ogg".into()));
        assert_eq!(asset.music, None);
        assert_eq!(asset.sound, Some("Combat pain male 3".into()));
    }
}
