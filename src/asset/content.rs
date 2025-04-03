use super::WINDOWS_SEPARATOR;
use crate::asset::sound::{SOUND_ATTR_FILEINBUFFER, SOUND_ATTR_FILENAME};
use crate::asset::text::{parse_text, TextContent};
use crate::{asset::ASSET_NODE_END, errors::AssetErrors, AssetBookmark, AssetType};
use anyhow::{anyhow, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use ddsfile::Dds;
use encoding_rs::WINDOWS_1252;
use std::io::{Read, Seek};

/// An enum listing all supported content types found on a database file
#[derive(Debug)]
pub enum AssetContent {
    /// A variant holding a OGG file
    Sound {
        filename: String,
        size: u32,
        bytes: Vec<u8>,
    },
    /// A variant holding a Direct Draw Surface
    Texture { width: u32, height: u32, dds: Dds },
    /// A variant holding a list of text components
    Text { contents: Vec<TextContent> },
    /// A variant indicating a not-supported content
    NotSupported,
}

impl AssetContent {
    /// Tries to convert a value to a variant of AssetContent
    pub fn read<T: Read + Seek>(reader: T, bookmark: &AssetBookmark) -> Result<AssetContent> {
        let content = match bookmark.asset_type {
            AssetType::Text => Self::read_text(reader, bookmark),
            AssetType::Sound | AssetType::Music => Self::read_sound(reader, bookmark),
            AssetType::Texture => Self::read_texture(reader, bookmark),
            _ => Ok(AssetContent::NotSupported),
        };
        content
    }

    /// Tries to convert a value to a AssetContent::Texture variant
    pub fn read_texture<T: Read + Seek>(
        mut reader: T,
        _bookmark: &AssetBookmark,
    ) -> Result<AssetContent> {
        let mut buffer = [0; 1];
        reader.read(&mut buffer)?;
        let [header_length] = buffer[..] else {
            return Err(anyhow!(AssetErrors::ParserError).context("texture header length"));
        };
        assert_eq!(header_length, 0x10);

        let mut buffer = [0; 16];
        reader.read(&mut buffer)?;
        let [_unknown1, width, height, _unknown2] = unsafe {
            let (_, values, _) = buffer.align_to::<u32>();
            values.try_into().unwrap()
        };

        // let pixels: u32 = height * length * 4;

        let mut buffer = [0; 1];
        reader.read(&mut buffer)?;
        let [unknown] = buffer[..] else {
            return Err(anyhow!(AssetErrors::ParserError).context("unknown texture value"));
        };
        assert_eq!(unknown, 0x64);

        // TODO: ???
        let mut buffer = [0; 48];
        reader.read(&mut buffer)?;

        let unknown_length = reader.read_u8()?;

        // TODO: ???
        let mut buffer = vec![0; unknown_length as usize];
        reader.read(&mut buffer)?;

        // DDS string starts here
        let dds = Dds::read(reader)?;

        Ok(AssetContent::Texture { width, height, dds })
    }

    /// Tries to parse content to a AssetContent::Sound variant
    pub fn read_sound<T: Read + Seek>(
        mut reader: T,
        bookmark: &AssetBookmark,
    ) -> Result<AssetContent> {
        let mut finished = false;
        let fail_pos = reader.stream_position().unwrap() + bookmark.size as u64 + 16u64;

        let mut filename: Option<String> = None;
        let mut size: Option<u32> = None;
        let mut bytes: Option<Vec<u8>> = None;

        while !finished {
            // fault tolerant check
            if reader.stream_position().unwrap() >= fail_pos {
                return Err(anyhow!(AssetErrors::ParserError).context("out of bounds"));
            }

            let attr_name_length = reader.read_u32::<LittleEndian>()?;

            // attr name
            let mut buffer = vec![0; attr_name_length as usize];
            reader.read(&mut buffer)?;
            let (attr_name, _, _) = WINDOWS_1252.decode(&buffer);
            let attr_name = attr_name.into_owned();

            let attr_flag = reader.read_u8()?;

            if attr_name == SOUND_ATTR_FILENAME {
                // attr_flag == 2
                assert_eq!(attr_flag, 0x2);

                let attr_value_length = reader.read_u32::<LittleEndian>()?;

                let mut buffer = vec![0; attr_value_length as usize];
                reader.read(&mut buffer)?;
                let (attr_value, _, _) = WINDOWS_1252.decode(&buffer);
                let attr_value = attr_value.into_owned();

                filename = Some(attr_value)
            } else if attr_name == SOUND_ATTR_FILEINBUFFER {
                // attr_flag == 4
                assert_eq!(attr_flag, 0x4);

                let content_size = reader.read_u32::<LittleEndian>()?;

                let mut content = vec![0; content_size as usize];
                reader.read(&mut content)?;

                size = Some(content_size);
                bytes = Some(content);
            }

            if filename.is_some() && bytes.is_some() {
                finished = true;
            }
        }

        if filename.is_some() && bytes.is_some() {
            return Ok(AssetContent::Sound {
                filename: filename.unwrap(),
                size: size.unwrap(),
                bytes: bytes.unwrap(),
            });
        }

        Ok(AssetContent::NotSupported)
    }

    /// Tries to parse content to a AssetContent::Text variant
    pub fn read_text<T: Read + Seek>(
        mut reader: T,
        _bookmark: &AssetBookmark,
    ) -> Result<AssetContent> {
        let mut finished = false;
        let mut contents: Vec<TextContent> = vec![];

        while !finished {
            let mut text_length: u32 = 0;
            let mut refs: Vec<String> = vec![];

            loop {
                let mut buffer = [0; 4];
                reader.read(&mut buffer)?;

                // check if we've reach the end
                let is_end = match String::from_utf8(buffer.into()) {
                    Err(_) => false,
                    Ok(value) => value == ASSET_NODE_END,
                };
                if is_end {
                    finished = true;
                    break;
                }

                // read as text length
                let length = unsafe {
                    let (_, values, _) = buffer.align_to::<u32>();
                    values[0]
                };

                // read string
                let mut buffer = vec![0; length as usize];
                reader.read(&mut buffer)?;
                let _ref = String::from_utf8(buffer)?;
                refs.push(_ref);

                // read next byte
                let value = reader.read_u8()?;
                assert_eq!(value, 0x2);

                // preview next len
                let mut buffer = [0; 4];
                reader.read(&mut buffer)?;

                if buffer != [0, 0, 0, 0] {
                    // check if this is the text length
                    text_length = unsafe {
                        let (_, values, _) = buffer.align_to::<u32>();
                        values[0]
                    };
                    break;
                }
            }

            if refs.len() == 0 {
                break;
            }

            // read content
            let mut buffer = vec![0; text_length as usize];
            reader.read(&mut buffer)?;
            let (content, _, _) = WINDOWS_1252.decode(&buffer);

            let sep = String::from_iter(WINDOWS_SEPARATOR);
            let parts: Vec<&str> = content.split(&sep).collect();

            let nodes = parts
                .iter()
                .filter(|x| **x != "")
                .map(|s| parse_text(s))
                .collect();

            let asset = TextContent { refs, nodes };

            // TODO: rich text examples
            // COLORS
            // "{{CFFFFFF}}Necrostacy"
            // "Activate {{#FFFF00}}Lucky Team{{#}}?"
            // "Premium item obtained and {{#FFFF00}}activated{{#}} successfully!")
            // "Tip: Press <{{CFFFFFF}}CTRL{{CBBBBBB}}> to begin combat mode"
            // VARIABLES
            // "You have received $count pieces of Ximerin!"
            // REFS
            // "This {1,2} is for me?"
            // "The realm wants you to eliminate {1,1} {1,2}."
            // CONDITIONS
            // "Introduction|gender=1" male
            // "Introduction|gender=0" female
            // "Greeting|level=34-45"

            contents.push(asset);
        }

        Ok(Self::Text { contents })
    }
}
