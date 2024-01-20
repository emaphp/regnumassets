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

### About

This crate provides a set of tools for retrieving information from a set of asset files that are located within your local installation of *Champions of Regnum*.

### Basic Usage

*Champions of Regnum* comes with 2 types of asset files: *index* files and *database* files. Both of these files are located in the game installation folder and use the `.idx` and `.sdb` extensions respectively. Each file contains a given set of asset files, that could be either sounds, music, textures, etc. For each index file there's a corresponding database files. Index files do not include the assets but provide information on how to retrieve a give asset from the Database file.

The process of retrieving data from asset files consist of parsing an index file using `ResourceIndex` to generate a list of bookmarks, each one pointing to a particular asset in the database file. Bookmarks can later be used to retrieve the asset data from the database file.

The next example illustrates how to retrieve the list of sounds from the corresponding index file:

```rust
use anyhow::Result;
use regnumassets::{AssetType, ResourceIndex};
use std::fs::File;

fn main() -> Result<()> {
    let f = File::open("data2.idx")?;
    let index = ResourceIndex::new(f).unwrap();

    let sounds = index.filter_by_type(AssetType::Sound);

    for sound in &sounds {
        println!(
            "Resource #{}: {}",
            sound.resource_id,
            sound.name.as_deref().unwrap_or("(unnamed)".into())
        );
    }

    Ok(())
}
```

The `ResourceIndex` struct provides an API to retrieve assets either by their resource id or by their asset type. Calling these methods will get you an instance of `AssetBookmark`. To get data from a database file we use the `AssetData` struct, which has a constructor expecting the database file handle and a `&AssetBookmark`.

The next example shows how to obain a sound by its id and save the contents to a new file:

```rust
use anyhow::Result;
use regnumassets::{AssetData, ResourceIndex};
use std::fs::File;
use std::io::Write;

fn main() -> Result<()> {
    let f = File::open("data2.idx")?;
    let index = ResourceIndex::new(f).unwrap();

    let sound = index.get_by_resource_id(50677).unwrap();

    let f = File::open("data2.sdb")?;
    let asset = AssetData::new(&f, &sound).unwrap();

    let filename = asset.filename.unwrap();
    println!("Writing file to {}", filename);

    let mut output = File::create(filename)?;
    output.write_all(asset.bytes.unwrap().as_ref())?;
    output.flush()?;

    Ok(())
}
```

### License

Released under the MIT License.

### Disclaimer

Champions of Regnum is a registered trademark of Nimble Giant Entertainment. I don't hold any type of relation to the company or its staff.
