/// An enum able to hold different variants of text content
#[derive(Debug, Clone)]
pub enum TextNode {
    /// The beginning of a list of text nodes
    Start,
    /// The end of a list of text nodes
    End,
    /// A number indicating a certain stage within a quest
    Stage(u32),
    /// A string indicating a topic/theme
    Topic(String),
    /// A free form text
    Content(String),
}

/// A wrapper holding a list of identifiers and text nodes
#[derive(Debug, Clone)]
pub struct TextContent {
    pub refs: Vec<String>,
    pub nodes: Vec<TextNode>,
}
