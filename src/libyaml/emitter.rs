use crate::libyaml::error::Error as LibyamlError;
use libyaml_safer::{self as sys, MappingStyle, ScalarStyle as SysScalarStyle, SequenceStyle};
use ouroboros::self_referencing;
use std::io;

#[derive(Debug)]
pub(crate) enum EmitError {
    Libyaml(LibyamlError),
}

#[self_referencing(pub_extras)]
pub(crate) struct Emitter {
    write: Box<dyn io::Write>,
    #[borrows(mut write)]
    #[not_covariant]
    emitter: sys::Emitter<'this>,
}

#[derive(Debug)]
pub(crate) enum Event<'a> {
    StreamStart,
    StreamEnd,
    DocumentStart,
    DocumentEnd,
    Scalar(Scalar<'a>),
    SequenceStart(Sequence),
    SequenceEnd,
    MappingStart(Mapping),
    MappingEnd,
}

#[derive(Debug)]
pub(crate) struct Scalar<'a> {
    pub tag: Option<String>,
    pub value: &'a str,
    pub style: ScalarStyle,
}

#[derive(Debug)]
pub(crate) enum ScalarStyle {
    Any,
    Plain,
    SingleQuoted,
    DoubleQuoted,
    Literal,
    Folded,
}

#[derive(Debug)]
pub(crate) struct Sequence {
    pub tag: Option<String>,
}

#[derive(Debug)]
pub(crate) struct Mapping {
    pub tag: Option<String>,
}

impl Emitter {
    pub fn create(write: Box<dyn io::Write>) -> Emitter {
        Emitter::new(write, |write| {
            let mut emitter = sys::Emitter::new();
            emitter.set_output(write.as_mut());
            emitter.set_unicode(true);
            emitter.set_width(-1);
            emitter
        })
    }

    pub fn emit(&mut self, event: Event) -> Result<(), EmitError> {
        let sys_event = match event {
            Event::StreamStart => sys::Event::stream_start(sys::Encoding::Utf8),
            Event::StreamEnd => sys::Event::stream_end(),
            Event::DocumentStart => sys::Event::document_start(None, &[], true),
            Event::DocumentEnd => sys::Event::document_end(true),
            Event::Scalar(scalar) => {
                let style = match scalar.style {
                    ScalarStyle::Any => SysScalarStyle::Any,
                    ScalarStyle::Plain => SysScalarStyle::Plain,
                    ScalarStyle::SingleQuoted => SysScalarStyle::SingleQuoted,
                    ScalarStyle::DoubleQuoted => SysScalarStyle::DoubleQuoted,
                    ScalarStyle::Literal => SysScalarStyle::Literal,
                    ScalarStyle::Folded => SysScalarStyle::Folded,
                };
                let tag = scalar.tag.as_deref();
                let plain_implicit = tag.is_none();
                let quoted_implicit = tag.is_none();
                sys::Event::scalar(
                    None,
                    tag,
                    scalar.value,
                    plain_implicit,
                    quoted_implicit,
                    style,
                )
            }
            Event::SequenceStart(sequence) => {
                let tag = sequence.tag.as_deref();
                let implicit = tag.is_none();
                sys::Event::sequence_start(None, tag, implicit, SequenceStyle::Any)
            }
            Event::SequenceEnd => sys::Event::sequence_end(),
            Event::MappingStart(mapping) => {
                let tag = mapping.tag.as_deref();
                let implicit = tag.is_none();
                sys::Event::mapping_start(None, tag, implicit, MappingStyle::Any)
            }
            Event::MappingEnd => sys::Event::mapping_end(),
        };

        self.with_emitter_mut(|emitter| {
            emitter
                .emit(sys_event)
                .map_err(|e| EmitError::Libyaml(LibyamlError::from_safer(e)))
        })?;

        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), EmitError> {
        self.with_emitter_mut(|emitter| {
            emitter
                .flush()
                .map_err(|e| EmitError::Libyaml(LibyamlError::from_safer(e)))
        })?;

        Ok(())
    }
}
