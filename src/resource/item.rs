use crate::errors::AssetErrors;
use anyhow::{anyhow, Result};
use encoding_rs::WINDOWS_1252;
use std::io::Read;

/// An index item represents the data structure used to locate an asset
pub struct ResourceIndexItem {
    pub uid: String,
    pub name: String,
    pub char_name: Option<String>,
    pub start: u32,
    pub unknown: u32,
    pub size: u32,
}

impl ResourceIndexItem {
    pub fn read<T: Read>(mut reader: T) -> Result<Self> {
        // uid length
        let mut buffer = [0; 2];
        reader.read(&mut buffer)?;
        let values = unsafe {
            let (_, values, _) = buffer.align_to::<u16>();
            values
        };
        let [uid_length] = values[..] else {
            return Err(anyhow!(AssetErrors::ParserError).context("index item uid length"));
        };

        // uid
        let mut buffer = vec![0; uid_length.into()];
        reader.read(&mut buffer)?;
        let uid = String::from_utf8(buffer)?;

        // TODO: ???
        let mut buffer = [0; 1];
        reader.read(&mut buffer)?;
        assert_eq!(buffer[0], 0);

        // node start position + size
        let mut buffer = [0; 4 * 3];
        reader.read(&mut buffer)?;
        let values = unsafe {
            let (_, values, _) = buffer.align_to::<u32>();
            values
        };
        let [start, unknown, size] = values[..] else {
            return Err(anyhow!(AssetErrors::ParserError).context("index item start/unknown/size"));
        };

        // name length
        let mut buffer = [0; 2];
        reader.read(&mut buffer)?;
        let values = unsafe {
            let (_, values, _) = buffer.align_to::<u16>();
            values
        };
        let [name_length] = values[..] else {
            return Err(anyhow!(AssetErrors::ParserError).context("index item name length"));
        };

        // name
        let mut buffer = vec![0; name_length.into()];
        reader.read(&mut buffer)?;
        let (name, _, _) = WINDOWS_1252.decode(&buffer);
        let name = name.into_owned();

        // characters.idx adds the char name here
        // play it "safe" and try to retrieve char name length
        let mut buffer = [0; 2];
        reader.read(&mut buffer)?;
        let values = unsafe {
            let (_, values, _) = buffer.align_to::<u16>();
            values
        };

        let char_name = match values[0] {
            0 => {
                // we still need to point at the end of the next node
                let mut buffer = [0; 2];
                reader.read(&mut buffer)?;
                None
            }
            len => {
                let mut buffer = vec![0; len.into()];
                reader.read(&mut buffer)?;
                let (char_name, _, _) = WINDOWS_1252.decode(&buffer);
                let char_name = char_name.into_owned();
                let mut buffer = [0; 1];
                reader.read(&mut buffer)?;
                Some(char_name)
            }
        };

        Ok(Self {
            uid,
            name,
            char_name,
            start,
            unknown,
            size,
        })
    }
}
