use super::AssetType;

#[derive(Debug, Clone)]
pub struct AssetBookmark {
    pub resource_id: u32,
    pub asset_type: AssetType,
    pub name: Option<String>,
    pub node_start: usize,
    pub node_end: usize,
    pub node_next: usize,
    pub size: u32,
}
