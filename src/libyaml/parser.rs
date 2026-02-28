use crate::libyaml::error::{Error, Mark, Result};
use crate::libyaml::tag::Tag;
use crate::libyaml::util::debug_lossy;
use libyaml_safer::{self as sys, EventData, ScalarStyle as SysScalarStyle};
use std::borrow::Cow;
use std::fmt::{self, Debug};
use std::io::{BufReader, Cursor};

pub(crate) struct Parser<'input> {
    inner: sys::Parser<BufReader<Box<dyn std::io::Read + 'input>>>,
    input_ref: Option<&'input [u8]>,
}

#[derive(Debug)]
pub(crate) enum Event<'input> {
    StreamStart,
    StreamEnd,
    DocumentStart,
    DocumentEnd,
    Alias(Anchor),
    Scalar(Scalar<'input>),
    SequenceStart(SequenceStart),
    SequenceEnd,
    MappingStart(MappingStart),
    MappingEnd,
}

pub(crate) struct Scalar<'input> {
    pub anchor: Option<Anchor>,
    pub tag: Option<Tag>,
    pub value: Box<[u8]>,
    pub style: ScalarStyle,
    pub repr: Option<&'input [u8]>,
}

#[derive(Debug)]
pub(crate) struct SequenceStart {
    pub anchor: Option<Anchor>,
    pub tag: Option<Tag>,
}

#[derive(Debug)]
pub(crate) struct MappingStart {
    pub anchor: Option<Anchor>,
    pub tag: Option<Tag>,
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct Anchor(Box<[u8]>);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum ScalarStyle {
    Plain,
    SingleQuoted,
    DoubleQuoted,
    Literal,
    Folded,
}

impl<'input> Parser<'input> {
    pub fn new(input: Cow<'input, [u8]>) -> Parser<'input> {
        let input_ref = match &input {
            Cow::Borrowed(slice) => Some(*slice),
            Cow::Owned(_) => None,
        };

        let reader: Box<dyn std::io::Read + 'input> = match input {
            Cow::Borrowed(slice) => Box::new(Cursor::new(slice)),
            Cow::Owned(vec) => Box::new(Cursor::new(vec)),
        };

        let buf_reader = BufReader::new(reader);
        let mut parser = sys::Parser::new();
        parser.set_input(buf_reader);
        parser.set_encoding(sys::Encoding::Utf8);
        Parser {
            inner: parser,
            input_ref,
        }
    }

    pub fn next(&mut self) -> Result<(Event<'input>, Mark)> {
        let event = self.inner.parse().map_err(Error::from_safer)?;
        let mark = Mark::from_safer(event.start_mark);
        let ret = convert_event(&event, self.input_ref);
        Ok((ret, mark))
    }
}

fn convert_event<'input>(event: &sys::Event, input_ref: Option<&'input [u8]>) -> Event<'input> {
    match &event.data {
        EventData::StreamStart { .. } => Event::StreamStart,
        EventData::StreamEnd => Event::StreamEnd,
        EventData::DocumentStart { .. } => Event::DocumentStart,
        EventData::DocumentEnd { .. } => Event::DocumentEnd,
        EventData::Alias { anchor } => Event::Alias(Anchor(Box::from(anchor.as_bytes()))),
        EventData::Scalar {
            anchor,
            tag,
            value,
            style,
            ..
        } => Event::Scalar(Scalar {
            anchor: anchor
                .as_ref()
                .map(|a: &String| Anchor(Box::from(a.as_bytes()))),
            tag: tag.as_ref().map(|t: &String| Tag(Box::from(t.as_bytes()))),
            value: Box::from(value.as_bytes()),
            style: match style {
                SysScalarStyle::Plain => ScalarStyle::Plain,
                SysScalarStyle::SingleQuoted => ScalarStyle::SingleQuoted,
                SysScalarStyle::DoubleQuoted => ScalarStyle::DoubleQuoted,
                SysScalarStyle::Literal => ScalarStyle::Literal,
                SysScalarStyle::Folded => ScalarStyle::Folded,
                _ => ScalarStyle::Plain,
            },
            repr: input_ref.and_then(|input| {
                let start = event.start_mark.index as usize;
                let end = event.end_mark.index as usize;
                if start <= end && end <= input.len() {
                    Some(&input[start..end])
                } else {
                    None
                }
            }),
        }),
        EventData::SequenceStart { anchor, tag, .. } => Event::SequenceStart(SequenceStart {
            anchor: anchor
                .as_ref()
                .map(|a: &String| Anchor(Box::from(a.as_bytes()))),
            tag: tag.as_ref().map(|t: &String| Tag(Box::from(t.as_bytes()))),
        }),
        EventData::SequenceEnd => Event::SequenceEnd,
        EventData::MappingStart { anchor, tag, .. } => Event::MappingStart(MappingStart {
            anchor: anchor
                .as_ref()
                .map(|a: &String| Anchor(Box::from(a.as_bytes()))),
            tag: tag.as_ref().map(|t: &String| Tag(Box::from(t.as_bytes()))),
        }),
        EventData::MappingEnd => Event::MappingEnd,
    }
}

impl Debug for Scalar<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let Scalar {
            anchor,
            tag,
            value,
            style,
            repr: _,
        } = self;

        struct LossySlice<'a>(&'a [u8]);

        impl Debug for LossySlice<'_> {
            fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                debug_lossy(self.0, formatter)
            }
        }

        formatter
            .debug_struct("Scalar")
            .field("anchor", anchor)
            .field("tag", tag)
            .field("value", &LossySlice(value))
            .field("style", style)
            .finish()
    }
}

impl Debug for Anchor {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        debug_lossy(&self.0, formatter)
    }
}
