use super::ASSET_NODE_END;
use crate::errors::AssetErrors;
use anyhow::{anyhow, Result};
use encoding_rs::WINDOWS_1252;
use std::io::{Read, Seek, SeekFrom};

/// Attribute for the name of an animation
pub const ASSET_ATTR_ANIMATION: &'static str = "ANIMATION";
/// Attribute for the name of a mesh
pub const ASSET_ATTR_MESH: &'static str = "MESH";
/// Attribute for the name of a material
pub const ASSET_ATTR_MATERIAL: &'static str = "MATERIAL";
/// Attribute for the name of a texture
pub const ASSET_ATTR_TEXTURE: &'static str = "TEXTURE";
/// Attribute for the name of a sound
pub const ASSET_ATTR_SOUND: &'static str = "SOUND";
/// Attribute for the name of a music track
pub const ASSET_ATTR_MUSIC: &'static str = "MUSIC";
/// Attribute for the asset filename
pub const ASSET_ATTR_FILENAME: &'static str = "filename";
/// Attribute for the asset binary
pub const ASSET_ATTR_FILE_IN_BUFFER: &'static str = "file_in_buffer";

/// An enum listing all possible attributes within an asset
#[derive(Debug, Clone)]
pub enum AssetAttribute {
    Blank,
    Start { resource_id: u32, unknown: u32 },
    End,
    Filename(String),
    Content { size: u32, bytes: Vec<u8> },
    Animation(String),
    Mesh(String),
    Material(String),
    Texture(String),
    Music(String),
    Sound(String),
}

/// Parses the content into an instance of AssetAttribute
pub fn read_attribute<T: Read + Seek>(mut reader: T) -> Result<AssetAttribute> {
    let mut buffer = [0; 4];
    reader.read(&mut buffer)?;
    let is_end = match String::from_utf8(buffer.into()) {
        Err(_) => false,
        Ok(value) => value == ASSET_NODE_END,
    };

    if is_end {
        return Ok(AssetAttribute::End);
    }

    if buffer == [0x0, 0x0, 0x0, 0x0] {
        reader.seek(SeekFrom::Current(12))?;
        return Ok(AssetAttribute::Blank);
    }

    let length = unsafe {
        let (_, values, _) = buffer.align_to::<u32>();
        values[0]
    };

    if length == 1 {
        let mut buffer = [0; 4];
        reader.read(&mut buffer)?;
        let resource_id = unsafe {
            let (_, values, _) = buffer.align_to::<u32>();
            values[0]
        };

        let mut buffer = [0; 4];
        reader.read(&mut buffer)?;
        let unknown = unsafe {
            let (_, values, _) = buffer.align_to::<u32>();
            values[0]
        };

        return Ok(AssetAttribute::Start {
            resource_id,
            unknown,
        });
    }

    // attribute name
    let mut buffer = vec![0; length as usize];
    reader.read(&mut buffer)?;
    let name = String::from_utf8(buffer).unwrap();

    return match name.as_str() {
        ASSET_ATTR_SOUND => {
            let mut buffer = [0; 4];
            reader.read(&mut buffer)?;
            let length = unsafe {
                let (_, values, _) = buffer.align_to::<u32>();
                values[0]
            };

            let mut buffer = vec![0; length as usize];
            reader.read(&mut buffer)?;
            let (name, _, _) = WINDOWS_1252.decode(&buffer);
            let sound = name.into_owned();
            Ok(AssetAttribute::Sound(sound))
        }
        ASSET_ATTR_MUSIC => {
            let mut buffer = [0; 4];
            reader.read(&mut buffer)?;
            let length = unsafe {
                let (_, values, _) = buffer.align_to::<u32>();
                values[0]
            };

            let mut buffer = vec![0; length as usize];
            reader.read(&mut buffer)?;
            let (name, _, _) = WINDOWS_1252.decode(&buffer);
            let music = name.into_owned();
            Ok(AssetAttribute::Music(music))
        }
        ASSET_ATTR_ANIMATION => {
            let mut buffer = [0; 4];
            reader.read(&mut buffer)?;
            let length = unsafe {
                let (_, values, _) = buffer.align_to::<u32>();
                values[0]
            };

            let mut buffer = vec![0; length as usize];
            reader.read(&mut buffer)?;
            let (name, _, _) = WINDOWS_1252.decode(&buffer);
            let animation = name.into_owned();
            Ok(AssetAttribute::Animation(animation))
        }
        ASSET_ATTR_MATERIAL => {
            let mut buffer = [0; 4];
            reader.read(&mut buffer)?;
            let length = unsafe {
                let (_, values, _) = buffer.align_to::<u32>();
                values[0]
            };

            let mut buffer = vec![0; length as usize];
            reader.read(&mut buffer)?;
            let (name, _, _) = WINDOWS_1252.decode(&buffer);
            let material = name.into_owned();
            Ok(AssetAttribute::Material(material))
        }
        ASSET_ATTR_MESH => {
            let mut buffer = [0; 4];
            reader.read(&mut buffer)?;
            let length = unsafe {
                let (_, values, _) = buffer.align_to::<u32>();
                values[0]
            };

            let mut buffer = vec![0; length as usize];
            reader.read(&mut buffer)?;
            let (name, _, _) = WINDOWS_1252.decode(&buffer);
            let mesh = name.into_owned();
            Ok(AssetAttribute::Mesh(mesh))
        }
        ASSET_ATTR_TEXTURE => {
            let mut buffer = [0; 4];
            reader.read(&mut buffer)?;
            let length = unsafe {
                let (_, values, _) = buffer.align_to::<u32>();
                values[0]
            };

            let mut buffer = vec![0; length as usize];
            reader.read(&mut buffer)?;
            let (name, _, _) = WINDOWS_1252.decode(&buffer);
            let texture = name.into_owned();
            Ok(AssetAttribute::Texture(texture))
        }
        ASSET_ATTR_FILENAME => {
            // TODO: ???
            let mut buffer = [0; 1];
            reader.read(&mut buffer)?;
            assert_eq!(buffer[0], 2);

            let mut buffer = [0; 4];
            reader.read(&mut buffer)?;
            let length = unsafe {
                let (_, values, _) = buffer.align_to::<u32>();
                values[0]
            };

            let mut buffer = vec![0; length as usize];
            reader.read(&mut buffer)?;
            let filename = String::from_utf8(buffer).unwrap();
            Ok(AssetAttribute::Filename(filename))
        }
        ASSET_ATTR_FILE_IN_BUFFER => {
            // TODO
            let mut buffer = [0; 1];
            reader.read(&mut buffer)?;
            let total = buffer[0];
            assert_eq!(total, 4);

            let mut buffer = vec![0; total as usize];
            reader.read(&mut buffer)?;
            let size = unsafe {
                let (_, values, _) = buffer.align_to::<u32>();
                values[0]
            };
            let mut bytes = vec![0; size as usize];
            reader.read(&mut bytes)?;
            Ok(AssetAttribute::Content { size, bytes })
        }
        attr => Err(anyhow!(AssetErrors::UnknownAttributeError(attr.into()))),
    };
}
