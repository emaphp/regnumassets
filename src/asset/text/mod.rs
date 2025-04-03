pub mod content;
pub mod parse;

pub use content::{TextContent, TextNode};
pub use parse::parse_text;

/// A string separator compatible with Windows systems
pub const WINDOWS_SEPARATOR: [char; 2] = [0x0D as char, 0x0A as char];
