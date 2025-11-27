use crate::libyaml;
use libyaml_safer as safer;
use std::io;

#[derive(Debug)]
pub(crate) enum Error {
    Libyaml(libyaml::error::Error),
    Io(io::Error),
}

pub(crate) struct Emitter<W>
where
    W: io::Write,
{
    events: Vec<Event>,
    buffer: Vec<u8>,
    last_flush_len: usize,
    writer: Option<W>,
    width: i32,
    indent: i32,
}

#[derive(Debug)]
pub(crate) enum Event {
    StreamStart,
    StreamEnd,
    DocumentStart,
    DocumentEnd,
    Scalar(Scalar),
    SequenceStart(Sequence),
    SequenceEnd,
    MappingStart(Mapping),
    MappingEnd,
}

#[derive(Debug)]
pub(crate) struct Scalar {
    pub tag: Option<String>,
    pub value: String,
    pub style: ScalarStyle,
}

#[derive(Debug)]
pub(crate) enum ScalarStyle {
    Any,
    Plain,
    SingleQuoted,
    Literal,
}

#[derive(Debug)]
pub(crate) struct Sequence {
    pub tag: Option<String>,
}

#[derive(Debug)]
pub(crate) struct Mapping {
    pub tag: Option<String>,
}

impl<W> Emitter<W>
where
    W: io::Write,
{
    pub fn new(write: W, width: i32, indent: i32) -> Emitter<W> {
        Emitter {
            events: Vec::new(),
            buffer: Vec::new(),
            last_flush_len: 0,
            writer: Some(write),
            width,
            indent,
        }
    }

    pub fn emit(&mut self, event: Event) -> Result<(), Error> {
        // Convert event to have 'static lifetime by allocating strings
        let static_event = match event {
            Event::StreamStart => Event::StreamStart,
            Event::StreamEnd => Event::StreamEnd,
            Event::DocumentStart => Event::DocumentStart,
            Event::DocumentEnd => Event::DocumentEnd,
            Event::Scalar(s) => Event::Scalar(Scalar {
                tag: s.tag,
                value: s.value.clone(),
                style: s.style,
            }),
            Event::SequenceStart(seq) => Event::SequenceStart(seq),
            Event::SequenceEnd => Event::SequenceEnd,
            Event::MappingStart(map) => Event::MappingStart(map),
            Event::MappingEnd => Event::MappingEnd,
        };
        self.events.push(static_event);

        // Flush immediately to emulate old behavior
        self.flush()
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        if !self.events.is_empty() {
            // Re-emit ALL events to regenerate complete output
            self.buffer.clear();
            let mut emitter = safer::Emitter::new();
            emitter.set_output_string(&mut self.buffer);
            emitter.set_unicode(true);
            emitter.set_width(self.width);
            emitter.set_indent(self.indent);

            for event in &self.events {
                let safe_event = convert_to_safer_event_ref(event);
                emitter
                    .emit(safe_event)
                    .map_err(|e| Error::Libyaml(libyaml::error::Error::from_safer_error(e)))?;
            }

            // Only write the new data since last flush
            if let Some(writer) = self.writer.as_mut() {
                if self.buffer.len() > self.last_flush_len {
                    writer
                        .write_all(&self.buffer[self.last_flush_len..])
                        .map_err(Error::Io)?;
                    writer.flush().map_err(Error::Io)?;
                    self.last_flush_len = self.buffer.len();
                }
            }
        }
        Ok(())
    }

    pub fn into_inner(mut self) -> Result<W, Error> {
        // Flush any remaining events first
        self.flush()?;

        self.writer.take().ok_or_else(|| {
            Error::Io(io::Error::new(
                io::ErrorKind::Other,
                "emitter writer missing",
            ))
        })
    }
}

fn convert_to_safer_event_ref(event: &Event) -> safer::Event {
    let safe_event = match event {
        Event::StreamStart => safer::Event::stream_start(safer::Encoding::Utf8),
        Event::StreamEnd => safer::Event::stream_end(),
        Event::DocumentStart => safer::Event::document_start(None, &[], true),
        Event::DocumentEnd => safer::Event::document_end(true),
        Event::Scalar(scalar) => {
            let tag_ref = scalar.tag.as_deref();
            let plain_implicit = scalar.tag.is_none();
            let quoted_implicit = scalar.tag.is_none();
            let style = match scalar.style {
                ScalarStyle::Any => safer::ScalarStyle::Any,
                ScalarStyle::Plain => safer::ScalarStyle::Plain,
                ScalarStyle::SingleQuoted => safer::ScalarStyle::SingleQuoted,
                ScalarStyle::Literal => safer::ScalarStyle::Literal,
            };
            safer::Event::scalar(
                None,
                tag_ref,
                &scalar.value,
                plain_implicit,
                quoted_implicit,
                style,
            )
        }
        Event::SequenceStart(sequence) => {
            let tag_ref = sequence.tag.as_deref();
            let implicit = sequence.tag.is_none();
            let style = safer::SequenceStyle::Any;
            safer::Event::sequence_start(None, tag_ref, implicit, style)
        }
        Event::SequenceEnd => safer::Event::sequence_end(),
        Event::MappingStart(mapping) => {
            let tag_ref = mapping.tag.as_deref();
            let implicit = mapping.tag.is_none();
            let style = safer::MappingStyle::Any;
            safer::Event::mapping_start(None, tag_ref, implicit, style)
        }
        Event::MappingEnd => safer::Event::mapping_end(),
    };

    safe_event
}
