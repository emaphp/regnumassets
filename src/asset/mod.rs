pub mod bookmark;
pub mod content;
pub mod data;
pub mod sound;
pub mod text;
pub mod texture;

pub use content::AssetContent;
pub use text::content::{TextContent, TextNode};
pub use text::WINDOWS_SEPARATOR;

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
/// A string used to identify an Image asset
pub const ASSET_TYPE_IMAGE: &'static str = "IMAGE";
/// A string used to identify a Text asset
pub const ASSET_TYPE_TEXT: &'static str = "TEXT";
/// A string used to identify a Binary asset
pub const ASSET_TYPE_BINARY: &'static str = "BINARY";
/// A string used to identify a Texture asset
pub const ASSET_TYPE_TEXTURE: &'static str = "TEXTURE";
/// A string used to identify a Font asset
pub const ASSET_TYPE_FONT: &'static str = "FONT";
/// A string used to identify an Effect asset
pub const ASSET_TYPE_EFFECT: &'static str = "EFFECT";
/// A string used to identify a Sound asset
pub const ASSET_TYPE_SOUND: &'static str = "SOUND";
/// A string used to identify a Music asset
pub const ASSET_TYPE_MUSIC: &'static str = "MUSIC";
/// A string used to identify a character mesh
pub const ASSET_TYPE_CHAR_MESH: &'static str = "mesh";
/// A string used to identify a MapObject asset
pub const ASSET_TYPE_MAPOBJECT: &'static str = "MAPOBJECT";
/// A string used to identify a TerrainRegion asset
pub const ASSET_TYPE_TERRAIN_REGION: &'static str = "TERRAIN_REGION";
/// A string used to identify a WorldMap asset
pub const ASSET_TYPE_WORLDMAP: &'static str = "WORLDMAP";
/// ???
pub const ASSET_TYPE_PCAUTH: &'static str = "pcauth";

#[derive(Debug, Clone, PartialEq)]
pub enum AssetType {
    Material,
    Animation,
    Mesh,
    Image,
    Text,
    Binary,
    Texture,
    Font,
    Effect,
    Music,
    Sound,
    Character,
    Auth,
    MapObject,
    TerrainRegion,
    WorldMap,
}

impl TryFrom<&str> for AssetType {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            ASSET_TYPE_MATERIAL => Ok(Self::Material),
            ASSET_TYPE_ANIMATION => Ok(Self::Animation),
            ASSET_TYPE_MESH => Ok(Self::Mesh),
            ASSET_TYPE_IMAGE => Ok(Self::Image),
            ASSET_TYPE_TEXT => Ok(Self::Text),
            ASSET_TYPE_BINARY => Ok(Self::Binary),
            ASSET_TYPE_TEXTURE => Ok(Self::Texture),
            ASSET_TYPE_EFFECT => Ok(Self::Effect),
            ASSET_TYPE_FONT => Ok(Self::Font),
            ASSET_TYPE_SOUND => Ok(Self::Sound),
            ASSET_TYPE_MUSIC => Ok(Self::Music),
            ASSET_TYPE_CHAR_MESH => Ok(Self::Character),
            ASSET_TYPE_MAPOBJECT => Ok(Self::MapObject),
            ASSET_TYPE_TERRAIN_REGION => Ok(Self::TerrainRegion),
            ASSET_TYPE_WORLDMAP => Ok(Self::WorldMap),
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
            Self::Image => ASSET_TYPE_IMAGE,
            Self::Text => ASSET_TYPE_TEXT,
            Self::Binary => ASSET_TYPE_BINARY,
            Self::Texture => ASSET_TYPE_TEXTURE,
            Self::Effect => ASSET_TYPE_EFFECT,
            Self::Font => ASSET_TYPE_FONT,
            Self::Sound => ASSET_TYPE_SOUND,
            Self::Music => ASSET_TYPE_MUSIC,
            Self::Character => ASSET_TYPE_CHAR_MESH,
            Self::MapObject => ASSET_TYPE_MAPOBJECT,
            Self::TerrainRegion => ASSET_TYPE_TERRAIN_REGION,
            Self::WorldMap => ASSET_TYPE_WORLDMAP,
            Self::Auth => ASSET_TYPE_PCAUTH,
        }
    }
}
