use super::item::ResourceIndexItem;
use super::node::ResourceIndexNode;
use crate::asset::{bookmark::AssetBookmark, AssetType};
use crate::asset::{ASSET_TYPE_CHAR_MESH, ASSET_TYPE_PCAUTH};
use crate::errors::AssetErrors;
use anyhow::{anyhow, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Read;

/// A struct representing the elements contained within a resource index file
pub struct ResourceIndex {
    pub bookmarks: Vec<AssetBookmark>,
}

impl ResourceIndex {
    pub fn read<T: Read>(mut reader: T) -> Result<Self> {
        // total nodes
        let mut buffer = [0; 4 * 3];
        reader.read(&mut buffer)?;
        let values = unsafe {
            let (_, values, _) = buffer.align_to::<u32>();
            values
        };

        let [_unknown_1, _unknown_2, total_nodes] = values[..] else {
            return Err(anyhow!(AssetErrors::ParserError).context("resource index total nodes"));
        };

        // parse header nodes
        let mut nodes = vec![];
        for _ in 0..total_nodes {
            let node = ResourceIndexNode::read(&mut reader)?;
            nodes.push(node);
        }

        // total items
        let total_items = reader.read_u32::<LittleEndian>()?;

        // parse body items
        let mut items = vec![];
        for _ in 0..total_items {
            let item = ResourceIndexItem::read(&mut reader)?;
            items.push(item);
        }

        // sort by start position
        nodes.sort_by(|a, b| a.node_start.cmp(&b.node_start));
        items.sort_by(|a, b| a.start.cmp(&b.start));

        let mut bookmarks = vec![];
        for i in 0..total_items {
            let node = &nodes[(i + 1) as usize];
            let node_start = node.node_start as usize;
            let node_end = node.node_end as usize;
            let node_next = node.node_next as usize;

            let item = &items[i as usize];

            // parse resource id
            let resource_id = {
                if let Some(pos) = item.uid.rfind(|c| c == '_') {
                    if let Ok(value) = item.uid[pos + 1..].parse::<u32>() {
                        Some(value)
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            let (name, asset_type): (Option<String>, AssetType) = {
                if ASSET_TYPE_CHAR_MESH == item.name {
                    (item.char_name.clone(), AssetType::Character)
                } else if item.uid == ASSET_TYPE_PCAUTH {
                    (Some(item.uid.clone()), AssetType::Auth)
                } else {
                    let parts: Vec<&str> = item.name.split("::").collect();
                    let asset_type = AssetType::try_from(parts[0])?;
                    (Some(parts[1].into()), asset_type)
                }
            };

            let size = item.size;

            // TODO
            assert_eq!(node_end - node_start, 23);

            bookmarks.push(AssetBookmark {
                resource_id,
                asset_type,
                name,
                node_start,
                node_end,
                node_next,
                size,
            });
        }

        // order by resource id
        bookmarks.sort_by(|a, b| a.resource_id.cmp(&b.resource_id));

        Ok(Self { bookmarks })
    }

    /// Retrieves an asset bookmark by its resource id
    pub fn get_by_resource_id(&self, resource_id: u32) -> Option<AssetBookmark> {
        match self
            .bookmarks
            .binary_search_by(|b| b.resource_id.cmp(&Some(resource_id)))
        {
            Ok(pos) => Some(self.bookmarks[pos].clone()),
            Err(_) => None,
        }
    }

    /// Retrieves a list of bookmarks by their asset type
    pub fn filter_by_type(&self, asset_type: AssetType) -> Vec<AssetBookmark> {
        self.bookmarks
            .iter()
            .filter(|b| b.asset_type == asset_type)
            .map(|b| b.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{AssetType, ResourceIndex};
    use std::fs::File;

    #[test]
    fn test_index_material() {
        let f = File::open("examples/regnum/data0.idx");
        assert!(f.is_ok());

        let index = ResourceIndex::read(f.unwrap()).unwrap();

        let found = index.get_by_resource_id(68070);
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.resource_id, Some(68070));
        assert_eq!(found.asset_type, AssetType::Material);
        assert_eq!(
            found.name,
            Some("matIgnis generales Cercas rota y vegetación".into())
        );
    }

    #[test]
    fn test_index_texture() {
        let f = File::open("examples/regnum/data1.idx");
        assert!(f.is_ok());

        let index = ResourceIndex::read(f.unwrap()).unwrap();

        let found = index.get_by_resource_id(1260);
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.resource_id, Some(1260));
        assert_eq!(found.asset_type, AssetType::Texture);
        assert_eq!(found.name, Some("Pradera gris demo".into()));
    }

    #[test]
    fn test_index_music() {
        let f = File::open("examples/regnum/data2.idx");
        assert!(f.is_ok());

        let index = ResourceIndex::read(f.unwrap()).unwrap();

        let found = index.get_by_resource_id(50194);
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.resource_id, Some(50194));
        assert_eq!(found.asset_type, AssetType::Music);
        assert_eq!(found.name, Some("Syrtis Music".into()));
    }

    #[test]
    fn test_index_text() {
        let f = File::open("examples/regnum/data5.idx");
        assert!(f.is_ok());

        let index = ResourceIndex::read(f.unwrap()).unwrap();

        let found = index.get_by_resource_id(59847);
        assert!(found.is_some());

        let found = found.unwrap();
        assert_eq!(found.resource_id, Some(59847));
        assert_eq!(found.asset_type, AssetType::Text);
        assert_eq!(found.name, Some("eng_faction_display_name".into()));
    }
}
