use super::parser::{FromData, Stream};

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tag(pub u32);

impl Tag {
    /// Creates a `Tag` from bytes.
    ///
    /// # Example
    ///
    /// ```text
    /// println!("{}", Tag::from_bytes(b"name"));
    /// ```
    #[inline]
    pub const fn from_bytes(bytes: &[u8; 4]) -> Self {
        Tag(((bytes[0] as u32) << 24)
            | ((bytes[1] as u32) << 16)
            | ((bytes[2] as u32) << 8)
            | (bytes[3] as u32))
    }

    /// Creates a `Tag` from bytes.
    ///
    /// In case of empty data will return `Tag` set to 0.
    ///
    /// When `bytes` are shorter than 4, will set missing bytes to ` `.
    ///
    /// Data after first 4 bytes is ignored.
    #[inline]
    pub fn from_bytes_lossy(bytes: &[u8]) -> Self {
        if bytes.is_empty() {
            return Tag::from_bytes(&[0, 0, 0, 0]);
        }

        let mut iter = bytes.iter().cloned().chain(core::iter::repeat(b' '));
        Tag::from_bytes(&[
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
        ])
    }

    /// Returns tag as 4-element byte array.
    #[inline]
    pub const fn to_bytes(self) -> [u8; 4] {
        [
            (self.0 >> 24 & 0xff) as u8,
            (self.0 >> 16 & 0xff) as u8,
            (self.0 >> 8 & 0xff) as u8,
            (self.0 >> 0 & 0xff) as u8,
        ]
    }

    /// Returns tag as 4-element byte array.
    #[inline]
    pub const fn to_chars(self) -> [char; 4] {
        [
            (self.0 >> 24 & 0xff) as u8 as char,
            (self.0 >> 16 & 0xff) as u8 as char,
            (self.0 >> 8 & 0xff) as u8 as char,
            (self.0 >> 0 & 0xff) as u8 as char,
        ]
    }

    /// Checks if tag is null / `[0, 0, 0, 0]`.
    #[inline]
    pub const fn is_null(&self) -> bool {
        self.0 == 0
    }

    /// Returns tag value as `u32` number.
    #[inline]
    pub const fn as_u32(&self) -> u32 {
        self.0
    }
}

impl core::fmt::Debug for Tag {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Tag({})", self)
    }
}

impl core::fmt::Display for Tag {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let b = self.to_chars();
        write!(
            f,
            "{}{}{}{}",
            b.get(0).unwrap_or(&' '),
            b.get(1).unwrap_or(&' '),
            b.get(2).unwrap_or(&' '),
            b.get(3).unwrap_or(&' ')
        )
    }
}

impl FromData for Tag {
    const SIZE: usize = 4;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        u32::parse(data).map(Tag)
    }
}

#[derive(Clone, Copy, Debug)]
#[allow(missing_docs)]
pub struct TableRecord {
    pub tag: Tag,
    #[allow(dead_code)]
    pub check_sum: u32,
    pub offset: u32,
    pub length: u32,
}

impl FromData for TableRecord {
    const SIZE: usize = 16;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(TableRecord {
            tag: s.read::<Tag>()?,
            check_sum: s.read::<u32>()?,
            offset: s.read::<u32>()?,
            length: s.read::<u32>()?,
        })
    }
}
