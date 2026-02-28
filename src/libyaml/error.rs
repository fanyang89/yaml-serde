use std::fmt::{self, Debug, Display};

pub(crate) type Result<T> = std::result::Result<T, Error>;

pub(crate) struct Error {
    inner: libyaml_safer::Error,
}

impl Error {
    pub(crate) fn from_safer(err: libyaml_safer::Error) -> Self {
        Error { inner: err }
    }

    pub fn mark(&self) -> Mark {
        Mark {
            inner: self.inner.problem_mark().unwrap_or_default(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.inner)
    }
}

impl Debug for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self.inner)
    }
}

#[derive(Copy, Clone, Default)]
pub(crate) struct Mark {
    inner: libyaml_safer::Mark,
}

impl Mark {
    pub(crate) fn from_safer(mark: libyaml_safer::Mark) -> Self {
        Mark { inner: mark }
    }

    pub fn index(&self) -> u64 {
        self.inner.index
    }

    pub fn line(&self) -> u64 {
        self.inner.line
    }

    pub fn column(&self) -> u64 {
        self.inner.column
    }
}

impl Display for Mark {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.inner.line != 0 || self.inner.column != 0 {
            write!(
                formatter,
                "line {} column {}",
                self.inner.line + 1,
                self.inner.column + 1,
            )
        } else {
            write!(formatter, "position {}", self.inner.index)
        }
    }
}

impl Debug for Mark {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let mut formatter = formatter.debug_struct("Mark");
        if self.inner.line != 0 || self.inner.column != 0 {
            formatter.field("line", &(self.inner.line + 1));
            formatter.field("column", &(self.inner.column + 1));
        } else {
            formatter.field("index", &self.inner.index);
        }
        formatter.finish()
    }
}
