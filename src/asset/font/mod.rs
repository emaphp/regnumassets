mod font;
mod parser;

use crate::asset::font::parser::Offset;
use crate::errors::AssetErrors;
use anyhow::{anyhow, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Seek};

#[derive(Clone)]
pub enum Font {
    /// Game bitmap font: zlib-compressed glyph atlas.
    ///
    /// `bytes` holds a complete zlib stream.
    FontBlob { bytes: Vec<u8>, height: u32 },
    TrueType(Vec<u8>),
    /// OpenType font using CFF outlines (magic 'OTTO')
    OpenType(Vec<u8>),
    // ttcf
    FontCollection { faces: u32, bytes: Vec<u8> },
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Magic {
    FontBlob,
    TrueType,
    OpenType,
    FontCollection,
}

impl parser::FromData for Magic {
    const SIZE: usize = 4;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        match u32::parse(data)? {
            0x09000000 | 0x0B000000 => Some(Magic::FontBlob),
            0x00010000 | 0x74727565 => Some(Magic::TrueType),
            0x4F54544F => Some(Magic::OpenType), // 'OTTO'
            0x74746366 => Some(Magic::FontCollection),
            _ => None,
        }
    }
}

fn _parse_magic(raw_data: &[u8]) -> Result<Magic> {
    let mut s = parser::Stream::new(raw_data);
    s.read::<Magic>()
        .ok_or(anyhow!(AssetErrors::ParserError).context("unknown font magic"))
}

fn parse_magic<T: Read + Seek>(mut reader: T) -> Result<Magic> {
    let mut buffer = vec![0; 4];
    reader.read(&mut buffer)?;
    let mut s = parser::Stream::new(&buffer);
    s.read::<Magic>()
        .ok_or(anyhow!(AssetErrors::ParserError).context("unknown font magic"))
}

pub fn parse_font<T: Read + Seek>(mut reader: T, size: u64) -> Result<Font> {
    // we don't know the size of this font yet
    // truetype fonts have variable size so we keep this position around
    let header_pos = reader.stream_position().unwrap();

    let magic = parse_magic(&mut reader)?;
    match magic {
        // check for font blob
        Magic::FontBlob => {
            reader.seek(std::io::SeekFrom::Current(-4))?; // go back to the "magic" number
            let fail_pos = reader.stream_position().unwrap() + size + 16u64;

            let mut bytes: Option<Vec<u8>> = None;
            let mut height: Option<u32> = None;

            loop {
                if reader.stream_position().unwrap() >= fail_pos {
                    return Err(anyhow!(AssetErrors::ParserError).context("out of bounds"));
                }

                // read attribute
                let attr_len = reader.read_u32::<LittleEndian>()?;
                let mut buffer = vec![0; attr_len as usize];
                reader.read(&mut buffer)?;
                let attr = String::from_utf8(buffer)?;

                match attr.as_str() {
                    "font_blob" => {
                        let mark = reader.read_u8()?;
                        assert_eq!(mark, 4);

                        let size = reader.read_u32::<LittleEndian>()?;
                        // the declared size is byte-exact: the zlib stream
                        // ends right at the next attribute boundary
                        let mut data = vec![0; size as usize];
                        reader.read(&mut data)?;
                        let _ = bytes.insert(data);
                    }
                    "font_height" => {
                        let _ = reader.read_u8()?;
                        let total = reader.read_u32::<LittleEndian>()?;
                        let _ = height.insert(total);
                    }
                    _ => {}
                }

                match (&bytes, height) {
                    (Some(_), Some(h)) => {
                        return Ok(Font::FontBlob {
                            bytes: bytes.unwrap(),
                            height: h,
                        })
                    }
                    _ => {}
                }
            }
        }
        // check for truetype font
        Magic::TrueType => {
            let bytes = read_sfnt(&mut reader, header_pos, size)?;
            return Ok(Font::TrueType(bytes));
        }
        // check for opentype font (CFF outlines)
        Magic::OpenType => {
            let bytes = read_sfnt(&mut reader, header_pos, size)?;
            return Ok(Font::OpenType(bytes));
        }
        // check for truetype font collection
        Magic::FontCollection => {
            let mut buffer = vec![0; (size - 4) as usize];
            reader.read(&mut buffer)?;
            // println!("size: {}", size);

            let mut s = parser::Stream::new(&buffer);
            s.skip::<u32>(); // version
            let faces = s
                .read::<u32>()
                .ok_or(anyhow!(AssetErrors::ParserError).context("malformed truetype font"))?;
            let offsets = s
                .read_array32::<parser::Offset32>(faces)
                .ok_or(anyhow!(AssetErrors::ParserError).context("malformed truetype font"))?;

            // determine the actual size of the collection: the end of the
            // furthest table across all faces (table offsets are relative
            // to the start of the collection)
            let mut font_size = s.offset() + 4; // we must add the 32-bit header

            for offset in offsets {
                let face_offset = offset
                    .to_usize()
                    .checked_sub(s.offset() + 4)
                    .ok_or(anyhow!(AssetErrors::ParserError).context("malformed truetype font"))?;
                s.advance_checked(face_offset)
                    .ok_or(anyhow!(AssetErrors::ParserError).context("malformed truetype font"))?;

                let magic = s
                    .read::<Magic>()
                    .ok_or(anyhow!(AssetErrors::ParserError).context("malformed truetype font"))?;
                // And face in a font collection can't be another collection.
                if magic == Magic::FontCollection {
                    return Err(
                        anyhow!(AssetErrors::ParserError).context("malformed truetype font")
                    );
                }
                let num_tables = s
                    .read::<u16>()
                    .ok_or(anyhow!(AssetErrors::ParserError).context("malformed truetype font"))?;
                s.advance(6); // searchRange (u16) + entrySelector (u16) + rangeShift (u16)
                let table_records = s
                    .read_array16::<font::TableRecord>(num_tables)
                    .ok_or(anyhow!(AssetErrors::ParserError).context("malformed truetype font"))?;

                for record in table_records {
                    if let Some(end) = record.offset.checked_add(record.length) {
                        font_size = font_size.max(end as usize);
                    }
                }
            }

            let font_size = font_size.min(size as usize);

            // read entire collection
            reader.seek(std::io::SeekFrom::Start(header_pos))?;
            let mut bytes = vec![0; font_size];
            reader.read_exact(&mut bytes)?;

            return Ok(Font::FontCollection { faces, bytes });
        }
        _ => {}
    }

    return Err(anyhow!(AssetErrors::ParserError).context("unknown font type magic"));
}

/// Reads a single-face sfnt font (TrueType or OpenType) starting at `header_pos`.
///
/// The actual font size is determined from its table directory: the end of the
/// furthest table (offsets are relative to the font start).
fn read_sfnt<T: Read + Seek>(
    reader: &mut T,
    header_pos: u64,
    asset_size: u64,
) -> Result<Vec<u8>> {
    // probe the table directory starting right after the 32-bit magic
    reader.seek(std::io::SeekFrom::Start(header_pos + 4))?;
    let mut buffer = vec![0; (asset_size - 4) as usize];
    reader.read_exact(&mut buffer)?;
    let mut s = parser::Stream::new(&buffer);
    let num_tables = s
        .read::<u16>()
        .ok_or(anyhow!(AssetErrors::ParserError).context("malformed font data"))?;
    s.advance(6); // searchRange (u16) + entrySelector (u16) + rangeShift (u16)
    let table_records = s
        .read_array16::<font::TableRecord>(num_tables)
        .ok_or(anyhow!(AssetErrors::ParserError).context("malformed font data"))?;

    let mut font_size = s.offset() + 4; // we must add the 32-bit header
    for record in table_records {
        if let Some(end) = record.offset.checked_add(record.length) {
            font_size = font_size.max(end as usize);
        }
    }
    let font_size = font_size.min(asset_size as usize);

    // read the font entirely
    reader.seek(std::io::SeekFrom::Start(header_pos))?;
    let mut bytes = vec![0; font_size];
    reader.read_exact(&mut bytes)?;
    Ok(bytes)
}
