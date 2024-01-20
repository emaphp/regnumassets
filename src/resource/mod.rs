pub mod index;
pub mod item;
pub mod node;

/// An enum listing the different types of assets contained on a single file
pub enum ResourceType {
    MeshesAnimationsMaterials,
    Textures,
    MusicSound,
}

/// An enum listing the different types of resource files
pub enum ResourceFormat {
    IndexFile,
    AssetDatabase,
}

/// Returns the filename containing the given resource type in the given format
pub fn get_resource_filename(res_type: ResourceType, res_format: ResourceFormat) -> String {
    format!(
        "{}.{}",
        match res_type {
            ResourceType::MeshesAnimationsMaterials => "data0",
            ResourceType::Textures => "data1",
            ResourceType::MusicSound => "data2",
        },
        match res_format {
            ResourceFormat::IndexFile => "idx",
            ResourceFormat::AssetDatabase => "sdb",
        }
    )
}
#[cfg(test)]
mod tests {
    use super::{get_resource_filename, ResourceFormat, ResourceType};

    #[test]
    fn test_index_filenames() {
        assert_eq!(
            get_resource_filename(
                ResourceType::MeshesAnimationsMaterials,
                ResourceFormat::IndexFile
            ),
            "data0.idx"
        );

        assert_eq!(
            get_resource_filename(ResourceType::Textures, ResourceFormat::IndexFile),
            "data1.idx"
        );

        assert_eq!(
            get_resource_filename(ResourceType::MusicSound, ResourceFormat::IndexFile),
            "data2.idx"
        );
    }

    #[test]
    fn test_database_filenames() {
        assert_eq!(
            get_resource_filename(
                ResourceType::MeshesAnimationsMaterials,
                ResourceFormat::AssetDatabase
            ),
            "data0.sdb"
        );

        assert_eq!(
            get_resource_filename(ResourceType::Textures, ResourceFormat::AssetDatabase),
            "data1.sdb"
        );

        assert_eq!(
            get_resource_filename(ResourceType::MusicSound, ResourceFormat::AssetDatabase),
            "data2.sdb"
        );
    }
}
