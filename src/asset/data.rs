use super::bookmark::AssetBookmark;
use super::{AssetType, ASSET_NODE_START};
use crate::AssetContent;
use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt};
use encoding_rs::WINDOWS_1252;
use std::io::{Read, Seek, SeekFrom};

/// A wrapper struct containing the data retrieved from the asset database file
pub struct AssetData {
    /// The asset type
    pub asset_type: AssetType,
    /// A string with the form 'resource_...'
    pub uid: String,
    /// A string with the form 'TYPE::NAME'
    pub resource_name: String,
    /// The asset name
    pub asset_name: String,
    /// An unique identifier
    pub resource_id: u32,
    /// The actual content
    pub content: AssetContent,

    // TODO
    _unknown: u32,
    _unknown2: [u8; 16],
    _unknown3: [u8; 16],
    _unknown4: u32,
    _maybe_size: u32,
}

impl AssetData {
    pub fn read<T: Read + Seek>(mut reader: T, bookmark: &AssetBookmark) -> Result<Self> {
        let pos = bookmark.node_end;
        reader.seek(SeekFrom::Start(pos as u64))?;

        // PAIR string
        let mut buffer = vec![0; 4];
        reader.read(&mut buffer)?;
        let node = String::from_utf8(buffer).unwrap();
        assert_eq!(node, ASSET_NODE_START);

        // TODO: ???
        let unknown = reader.read_u32::<LittleEndian>()?;

        // uid length
        let uid_length = reader.read_u8()?;

        // TODO: ???
        let mut unknown2 = [0; 16];
        reader.read(&mut unknown2)?;

        // uid
        let mut buffer = vec![0; uid_length as usize];
        reader.read(&mut buffer)?;
        let uid = String::from_utf8(buffer)?;

        // resource name length
        let resource_name_length = reader.read_u8()?;

        // resource name
        let mut buffer = vec![0; resource_name_length as usize];
        reader.read(&mut buffer)?;
        let (resource_name, _, _) = WINDOWS_1252.decode(&buffer);
        let resource_name = resource_name.into_owned();

        // separator
        let mut buffer = [0; 4];
        reader.read(&mut buffer)?;
        assert_eq!(buffer, [0, 0, 0, 0]);

        // TODO: size?
        let maybe_size = reader.read_u32::<LittleEndian>()?;
        assert_eq!(maybe_size, bookmark.size);

        // TODO: ???
        let mut unknown3 = [0; 16];
        reader.read(&mut unknown3)?;

        // TODO: ???
        let mut buffer = [0; 4];
        reader.read(&mut buffer)?;
        assert_eq!(buffer, [1, 0, 0, 0]);

        // resource id
        let resource_id = reader.read_u32::<LittleEndian>()?;

        // TODO: ???
        let unknown4 = reader.read_u32::<LittleEndian>()?;

        // asset type length
        let asset_type_length = reader.read_u32::<LittleEndian>()?;

        // asset type
        let mut buffer = vec![0; asset_type_length as usize];
        reader.read(&mut buffer)?;

        // asset name length
        let asset_name_length = reader.read_u32::<LittleEndian>()?;

        // asset name
        let mut buffer = vec![0; asset_name_length as usize];
        reader.read(&mut buffer)?;
        let (asset_name, _, _) = WINDOWS_1252.decode(&buffer);
        let asset_name = asset_name.into_owned();

        // TODO: ???
        let mut buffer = [0; 16];
        reader.read(&mut buffer)?;

        let content = AssetContent::read(reader, bookmark)?;

        Ok(AssetData {
            asset_type: bookmark.asset_type.clone(),
            uid,
            resource_id,
            resource_name,
            asset_name,
            content,
            _unknown: unknown,
            _unknown2: unknown2,
            _unknown3: unknown3,
            _unknown4: unknown4,
            _maybe_size: maybe_size,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{AssetContent, AssetData, AssetType, ResourceIndex};
    use std::fs::File;

    #[test]
    fn test_music_database() {
        let f = File::open("examples/regnum/data2.idx");
        assert!(f.is_ok());

        let index = ResourceIndex::read(f.unwrap()).unwrap();
        let music = index.get_by_resource_id(56934).unwrap();
        let sound = index.get_by_resource_id(50677).unwrap();

        let f = File::open("examples/regnum/data2.sdb");
        assert!(f.is_ok());
        let f = f.unwrap();

        let asset = AssetData::read(&f, &music).unwrap();
        assert_eq!(asset.asset_type, AssetType::Music);
        assert_eq!(asset.resource_id, 56934);

        match asset.content {
            AssetContent::Sound {
                filename,
                bytes: _b,
                size: _s,
            } => {
                assert_eq!(filename, ("regnum_ignis.ogg"));
            }
            _ => {
                //
            }
        }

        let asset = AssetData::read(&f, &sound).unwrap();
        assert_eq!(asset.asset_type, (AssetType::Sound));
        assert_eq!(asset.resource_id, 50677);

        match asset.content {
            AssetContent::Sound {
                filename,
                bytes: _b,
                size: _s,
            } => {
                assert_eq!(filename, ("combat_pain_male_3.ogg"));
            }
            _ => {
                //
            }
        }
    }
}
