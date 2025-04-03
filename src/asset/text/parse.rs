use super::TextNode;
use nom::{
    bytes::complete::{is_not, take_while},
    character::complete::char,
    sequence::delimited,
    AsChar, IResult, Parser,
};

/// A string indicating the start of a sequence of text nodes
pub const TEXT_START: &str = "[start]";
/// A string indicating the end of a sequence of text nodes
pub const TEXT_END: &str = "[end]";

/// A parser function able to identify topics within a text
pub fn delimited_text(input: &str) -> IResult<&str, &str> {
    delimited(char('['), is_not("]"), char(']')).parse(input)
}

/// A helper function able to consume digits fron a string
fn while_digit(s: &str) -> IResult<&str, &str> {
    take_while(AsChar::is_dec_digit)(s)
}

/// A parser function able to identify steps within a string
pub fn delimited_digit(input: &str) -> IResult<&str, &str> {
    delimited(char('['), while_digit, char(']')).parse(input)
}

/// Parses a text into a variant of TextContent
pub fn parse_text(text: &str) -> TextNode {
    if text.trim() == TEXT_START {
        return TextNode::Start;
    }

    if text.trim() == TEXT_END {
        return TextNode::End;
    }

    match delimited_digit(text) {
        Ok((_, d)) => TextNode::Stage(d.parse().unwrap()),
        Err(_) => match delimited_text(text) {
            Ok((_, t)) => TextNode::Topic(t.into()),
            Err(_) => TextNode::Content(text.to_string()),
        },
    }
}
