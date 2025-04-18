A crate for parsing game asset files from MMORPG [Champions of Regnum](https://www.championsofregnum.com/).

[![regnumassets on Crates.io](https://img.shields.io/crates/v/regnumassets.svg?color=brightgreen)](https://crates.io/crates/regnumassets)
[![Documentation](https://img.shields.io/docsrs/regnumassets/latest.svg)](https://docs.rs/regnumassets)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/emaphp/regnumassets/blob/master/LICENSE)
----------------

Table of Contents
=================

* [About](#about)
* [Basic Usage](#basic-usage)
* [License](#license)
* [Disclaimer](#disclaimer)

### About ###

This crate provides a set of tools for parsing/extracting asset files from your local installation of *Champions of Regnum*.

### Basic Usage ###

*Champions of Regnum* comes with 2 types of asset files: *index* files and *database* files. Both of these files are located in the game installation folder and use the `.idx` and `.sdb` extensions respectively. Each file contains a given set of asset files, that could be either sounds, music, textures, etc. For each index file there's a corresponding database files. Index files do not include the assets but provide information on how to retrieve a given asset from the database file.

The process of retrieving data from asset files consist of parsing an index file using `ResourceIndex` to generate a list of bookmarks. Each bookmark points to a particular asset in the database file. Bookmarks keep track of where in the database file an asset is located, allowing for parsing and extracting their contents.

The next example shows how to retrieve the list of sounds from the corresponding index file:

```rust
use anyhow::Result;
use regnumassets::{AssetType, ResourceIndex};
use std::fs::File;

fn main() -> Result<()> {
    let f = File::open("data2.idx")?;
    let index = ResourceIndex::read(f).unwrap();

    let sounds = index.filter_by_type(AssetType::Sound);

    for sound in &sounds {
        println!(
            "Resource #{}: {}",
            sound.resource_id.unwrap_or(0),
            sound.name.as_deref().unwrap_or("(unnamed)".into())
        );
    }

    Ok(())
}
```

The `AssetType` enum defines the following variants:

```rust
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
}
```

### Asset data ###

The `ResourceIndex` struct provides an API for retrieving  assets either by their resource id or by their asset type. Calling these methods will get you an instance of `AssetBookmark`. To get data from a database file we use the `AssetData` struct, which has a constructor expecting the database file handle and a `&AssetBookmark`.

```rust
fn main() -> Result<()> {
    let f = File::open("data2.idx")?;
    let index = ResourceIndex::read(f).unwrap();

    let sound = index.get_by_resource_id(50677).unwrap();

    let f = File::open("data2.sdb")?;
    let asset = AssetData::read(&f, &sound).unwrap();

    // ...
}
```

The `AssetData` struct includes a `content` property that contains the actual asset. This enum type defines a variant for each supported type. Types that are not currently supported will always generate a value of type `AssetContent::NotSupported`.

```rust
pub enum AssetContent {
    /// A variant holding an Ogg Vorbis file
    Sound {
        filename: String,
        size: u32,
        bytes: Vec<u8>,
    },
    /// A variant holding a Direct Draw Surface
    Texture { width: u32, height: u32, dds: Dds },
    /// A variant holding a list of text components
    Text { contents: Vec<TextContent> },
    /// A variant holding a JPEG image
    Image { bytes: Vec<u8> },
    /// A variant indicating a not-supported content
    NotSupported,
}
```

#### Text ####

Within the game, a text can either be used for sinple translation or to build complex quest scripts. To cover all these different cases, texts are parsed into a list of `TextContent`:

```rust
pub struct TextContent {
    pub refs: Vec<String>,
    pub nodes: Vec<TextNode>,
}
```

All text components include a list of numerical identifiers (`refs`). These `refs` are used to identify a node and contain at least one element. Each node also contains a list of string elements of type `TextNode`. This enum is able to identify cases where a text refers to a topic or a particular quest stage:

```rust
pub enum TextNode {
    /// The beginning of a list of text nodes
    Start,
    /// The end of a list of text nodes
    End,
    /// A number indicating a stage in a quest
    Stage(u32),
    /// A string indicating a topic/theme
    Topic(String),
    /// A free form text
    Content(String),
}
```

This next example parses `eng_npc_template_dialog`, a resource including NPC dialog from the game:

```rust
use anyhow::Result;
use regnumassets::{ResourceIndex, AssetData, AssetContent};
use std::fs::File;

fn main() -> Result<()> {
    let f = File::open("examples/regnum/data5.idx")?;
    let index = ResourceIndex::read(f).unwrap();

    let text = index.get_by_resource_id(51277).unwrap();

    let f = File::open("examples/regnum/data5.sdb")?;
    let asset = AssetData::read(&f, &text).unwrap();

    match asset.content {
        AssetContent::Text { contents } => {
            for content in contents {
                println!("refs: {:?}", content.refs);
                for node in &content.nodes {
                    println!("TEXT: {:?}", node);
                }
            }
        }
        _ => {
            println!("could not read text asset")
        }
    }

    Ok(())
}
```

Keep in mind that regular text can also include format elements to change font color and such. This library does not provide features for post-processing these type of strings.

#### Sound ####

Both music and sound are stored using the Ogg Vorbis format. The `bytes` attribute will include the raw data. A `filename` attribute is also included.

The next example shows how to obain a sound by its id and save the contents to a new file:

```rust
use anyhow::Result;
use regnumassets::{ResourceIndex, AssetData, AssetContent};
use std::fs::File;
use std::io::Write;

fn main() -> Result<()> {
    let f = File::open("data2.idx")?;
    let index = ResourceIndex::read(f).unwrap();

    let sound = index.get_by_resource_id(50677).unwrap();

    let f = File::open("data2.sdb")?;
    let asset = AssetData::read(&f, &sound).unwrap();

    match asset.content {
        AssetContent::Sound {
            bytes,
            filename,
            size,
        } => {
            println!("writing {} bytes file to {}", size, filename);

            let mut output = File::create(filename)?;
            output.write_all(bytes.as_ref())?;
            output.flush()?;
        }
        _ => {
            println!("couuld not parse sound asset")
        }
    }

    Ok(())
}
```

#### Texture ####

Textures are stored using the [DirectDraw Surface](https://en.wikipedia.org/wiki/DirectDraw_Surface) format. In order to parse these assets, the [ddsfile::Dds](https://docs.rs/ddsfile/latest/ddsfile/struct.Dds.html) struct is used. This struct can then be used to export the contents to a `.dds` file.

```rust
use anyhow::Result;
use regnumassets::{ResourceIndex, AssetData, AssetContent};
use std::fs::File;
use std::io::Write;

fn main() -> Result<()> {
    let f = File::open("examples/regnum/data6.idx")?;
    let index = ResourceIndex::read(f).unwrap();

    let texture = index.get_by_resource_id(85953).unwrap();

    let f = File::open("examples/regnum/data6.sdb")?;
    let asset = AssetData::read(&f, &texture).unwrap();

    match asset.content {
        AssetContent::Texture { width, height, dds } => {
            println!(
                "writing texture '{}' ({}x{}) to out.dds",
                asset.asset_name, width, height
            );

            let mut file = File::create("out.dds")?;
            dds.write(&mut file)?;
            file.flush()?;
        }
        _ => {
            println!("this content is not supported")
        }
    }

    Ok(())
}
```

#### Image ####

Images are stored using the `JPEG` (`JFIF`) format. These assets are exported as slices of bytes.

```rust
use anyhow::Result;
use regnumassets::{AssetContent, AssetData, ResourceIndex};
use std::fs::File;
use std::io::Write;

fn main() -> Result<()> {
    let f = File::open("data5.idx")?;
    let index = ResourceIndex::read(f).unwrap();

    let image = index.get_by_resource_id(75879).unwrap();

    let f = File::open("data5.sdb")?;
    let asset = AssetData::read(&f, &image).unwrap();

    match asset.content {
        AssetContent::Image { bytes } => {
            let filename = "out.jpeg";
            println!("writing file to {}", filename);
            let mut output = File::create(filename)?;
            output.write_all(bytes.as_ref())?;
            output.flush()?;
        }
        _ => {
            println!("couuld not parse image asset")
        }
    }

    Ok(())
}
```

### License ###

Released under the MIT License.

### Disclaimer ###

Champions of Regnum is a registered trademark of Nimble Giant Entertainment. I don't hold any type of relation to the company or its staff.
