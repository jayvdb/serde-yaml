use crate::libyaml::error::{Error, Mark, Result};
use crate::libyaml::tag::Tag;
use libyaml_safer as safer;
use std::borrow::Cow;
use std::fmt::{self, Debug};

pub(crate) struct Parser<'input> {
    inner: safer::Parser<'input>,
}

#[derive(Debug)]
pub(crate) enum Event {
    StreamStart,
    StreamEnd,
    DocumentStart,
    DocumentEnd,
    Alias(Anchor),
    Scalar(Scalar),
    SequenceStart(SequenceStart),
    SequenceEnd,
    MappingStart(MappingStart),
    MappingEnd,
}

pub(crate) struct Scalar {
    pub anchor: Option<Anchor>,
    pub tag: Option<Tag>,
    pub value: Box<[u8]>,
    pub style: ScalarStyle,
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
        let mut inner = safer::Parser::new();
        let input_slice: &'input [u8] = Box::leak(input.into_owned().into_boxed_slice());
        let input_ref: &'input mut &'input [u8] = Box::leak(Box::new(input_slice));
        inner.set_input_string(input_ref);
        inner.set_encoding(safer::Encoding::Utf8);
        Parser { inner }
    }

    pub fn next(&mut self) -> Result<(Event, Mark)> {
        let event = self.inner.parse().map_err(Error::from_safer_error)?;

        let mark = Mark::from_safer_mark(event.start_mark);
        let converted = convert_event(&event);

        Ok((converted, mark))
    }
}

fn convert_event(event: &safer::Event) -> Event {
    use safer::EventData;

    match &event.data {
        EventData::StreamStart { .. } => Event::StreamStart,
        EventData::StreamEnd => Event::StreamEnd,
        EventData::DocumentStart { .. } => Event::DocumentStart,
        EventData::DocumentEnd { .. } => Event::DocumentEnd,
        EventData::Alias { anchor } => {
            Event::Alias(Anchor(anchor.as_bytes().to_vec().into_boxed_slice()))
        }
        EventData::Scalar {
            anchor,
            tag,
            value,
            style,
            ..
        } => Event::Scalar(Scalar {
            anchor: anchor
                .as_ref()
                .map(|a| Anchor(a.as_bytes().to_vec().into_boxed_slice())),
            tag: tag
                .as_ref()
                .map(|t| Tag(t.as_bytes().to_vec().into_boxed_slice())),
            value: value.as_bytes().to_vec().into_boxed_slice(),
            style: match style {
                safer::ScalarStyle::Plain => ScalarStyle::Plain,
                safer::ScalarStyle::SingleQuoted => ScalarStyle::SingleQuoted,
                safer::ScalarStyle::DoubleQuoted => ScalarStyle::DoubleQuoted,
                safer::ScalarStyle::Literal => ScalarStyle::Literal,
                safer::ScalarStyle::Folded => ScalarStyle::Folded,
                _ => ScalarStyle::Plain,
            },
        }),
        EventData::SequenceStart { anchor, tag, .. } => Event::SequenceStart(SequenceStart {
            anchor: anchor
                .as_ref()
                .map(|a| Anchor(a.as_bytes().to_vec().into_boxed_slice())),
            tag: tag
                .as_ref()
                .map(|t| Tag(t.as_bytes().to_vec().into_boxed_slice())),
        }),
        EventData::SequenceEnd => Event::SequenceEnd,
        EventData::MappingStart { anchor, tag, .. } => Event::MappingStart(MappingStart {
            anchor: anchor
                .as_ref()
                .map(|a| Anchor(a.as_bytes().to_vec().into_boxed_slice())),
            tag: tag
                .as_ref()
                .map(|t| Tag(t.as_bytes().to_vec().into_boxed_slice())),
        }),
        EventData::MappingEnd => Event::MappingEnd,
    }
}

impl Debug for Scalar {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let Scalar {
            anchor,
            tag,
            value,
            style,
        } = self;

        struct LossySlice<'a>(&'a [u8]);

        impl Debug for LossySlice<'_> {
            fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "{:?}", String::from_utf8_lossy(self.0))
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
        write!(formatter, "{:?}", String::from_utf8_lossy(&self.0))
    }
}
