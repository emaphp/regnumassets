/// An enum that identifies possibles causes of errors during asset file parsing
#[derive(Debug)]
pub enum AssetErrors {
    ParserError,
    UnknownAssetTypeError(String),
    UnknownAttributeError(String),
}

impl std::fmt::Display for AssetErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParserError => write!(f, "Could not align data to the the expected format"),
            Self::UnknownAssetTypeError(asset_type) => {
                write!(f, "unknown asset type: {}", asset_type)
            }
            Self::UnknownAttributeError(attr) => {
                write!(f, "unknown asset attribute: {}", attr)
            }
        }
    }
}

impl std::error::Error for AssetErrors {}
