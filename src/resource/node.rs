use crate::errors::AssetErrors;
use anyhow::{anyhow, Context, Result};
use std::io::Read;

/// A wrapper struct representing a single asset node located in the resource index header
#[derive(Debug)]
pub struct ResourceIndexNode {
    pub node_start: u32,
    pub node_type: u32,
    pub node_next: u32,
    pub node_previous: u32,
    pub node_end: u32,
}

impl ResourceIndexNode {
    pub fn read<T: Read>(mut reader: T) -> Result<Self> {
        let mut buffer = [0; 4 * 5];
        reader.read(&mut buffer)?;

        let values = unsafe {
            let (_, values, _) = buffer.align_to::<u32>();
            values
        };

        let [node_start, node_type, node_next, node_previous, node_end] = values[..] else {
            return Err(anyhow!(AssetErrors::ParserError)).context("index node");
        };

        Ok(Self {
            node_start,
            node_type,
            node_next,
            node_previous,
            node_end,
        })
    }
}
