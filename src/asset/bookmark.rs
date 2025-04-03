use super::AssetType;

/// An wrapper struct pointing to a specific asset in a database file
#[derive(Debug, Clone)]
pub struct AssetBookmark {
    pub resource_id: Option<u32>,
    pub asset_type: AssetType,
    pub name: Option<String>,
    pub node_start: usize,
    pub node_end: usize,
    pub node_next: usize,
    pub size: u32,
}
