pub mod attribute;
pub mod bookmark;
pub mod data;

use crate::errors::AssetErrors;

/// Marks the start of the asset node
pub const ASSET_NODE_START: &'static str = "PAIR";
/// Marks the end of the asset node
pub const ASSET_NODE_END: &'static str = "RIAP";

/// A string used to identify a Material asset
pub const ASSET_TYPE_MATERIAL: &'static str = "MATERIAL";
/// A string used to identify an Animation asset
pub const ASSET_TYPE_ANIMATION: &'static str = "ANIMATION";
/// A string used to identify a Mesh asset
pub const ASSET_TYPE_MESH: &'static str = "MESH";
/// A string used to identify a Texture asset
pub const ASSET_TYPE_TEXTURE: &'static str = "TEXTURE";
/// A string used to identify a Sound asset
pub const ASSET_TYPE_SOUND: &'static str = "SOUND";
/// A string used to identify a Music asset
pub const ASSET_TYPE_MUSIC: &'static str = "MUSIC";

#[derive(Debug, Clone, PartialEq)]
pub enum AssetType {
    Material,
    Animation,
    Mesh,
    Texture,
    Music,
    Sound,
}

impl TryFrom<&str> for AssetType {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            ASSET_TYPE_MATERIAL => Ok(Self::Material),
            ASSET_TYPE_ANIMATION => Ok(Self::Animation),
            ASSET_TYPE_MESH => Ok(Self::Mesh),
            ASSET_TYPE_TEXTURE => Ok(Self::Texture),
            ASSET_TYPE_SOUND => Ok(Self::Sound),
            ASSET_TYPE_MUSIC => Ok(Self::Music),
            unknown => Err(anyhow::anyhow!(AssetErrors::UnknownAssetTypeError(
                unknown.to_owned()
            ))),
        }
    }
}

impl Into<&str> for AssetType {
    fn into(self) -> &'static str {
        match self {
            Self::Material => ASSET_TYPE_MATERIAL,
            Self::Animation => ASSET_TYPE_ANIMATION,
            Self::Mesh => ASSET_TYPE_MESH,
            Self::Texture => ASSET_TYPE_TEXTURE,
            Self::Sound => ASSET_TYPE_SOUND,
            Self::Music => ASSET_TYPE_MUSIC,
        }
    }
}
