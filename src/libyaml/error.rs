use libyaml_safer as safer;
use std::fmt::{self, Debug, Display};

pub(crate) type Result<T> = std::result::Result<T, Error>;

pub(crate) struct Error {
    message: String,
    mark: Mark,
}

impl Error {
    pub fn from_safer_error(error: safer::Error) -> Self {
        let error_string = error.to_string();

        // Parse the safer error format to extract components and reformat to match old unsafe-libyaml format
        // New format: "Scanner error: line 2 column 1: found character... while scanning... (line 2 column 1)"
        // Old format: "found character... at line 2 column 1, while scanning..."
        let message = if let Some(reformatted) = reformat_error_message(&error_string) {
            reformatted
        } else {
            error_string
        };

        Error {
            message,
            mark: Mark::default(),
        }
    }

    pub fn mark(&self) -> Mark {
        self.mark
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.message)?;
        if self.mark.line != 0 || self.mark.column != 0 {
            write!(formatter, " at {}", self.mark)?;
        } else if self.mark.index != 0 {
            write!(formatter, " at position {}", self.mark.index)?;
        }
        Ok(())
    }
}

impl Debug for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let mut formatter = formatter.debug_struct("Error");
        formatter.field("message", &self.message);
        if self.mark.line != 0 || self.mark.column != 0 {
            formatter.field("mark", &self.mark);
        } else if self.mark.index != 0 {
            formatter.field("index", &self.mark.index);
        }
        formatter.finish()
    }
}

#[derive(Copy, Clone, Default)]
pub(crate) struct Mark {
    index: usize,
    line: usize,
    column: usize,
}

impl Mark {
    pub fn from_safer_mark(mark: safer::Mark) -> Self {
        Self {
            index: mark.index as usize,
            line: mark.line as usize,
            column: mark.column as usize,
        }
    }

    pub fn index(&self) -> u64 {
        self.index as u64
    }

    pub fn line(&self) -> u64 {
        self.line as u64
    }

    pub fn column(&self) -> u64 {
        self.column as u64
    }
}

impl Display for Mark {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.line != 0 || self.column != 0 {
            write!(
                formatter,
                "line {} column {}",
                self.line + 1,
                self.column + 1,
            )
        } else {
            write!(formatter, "position {}", self.index)
        }
    }
}

impl Debug for Mark {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let mut formatter = formatter.debug_struct("Mark");
        if self.line != 0 || self.column != 0 {
            formatter.field("line", &(self.line + 1));
            formatter.field("column", &(self.column + 1));
        } else {
            formatter.field("index", &self.index);
        }
        formatter.finish()
    }
}

/// Reformat libyaml-safer error messages to match the old unsafe-libyaml format
///
/// New format: "Scanner error: line 2 column 1: found character... while scanning... (line 2 column 1)"
/// Old format: "found character... at line 2 column 1, while scanning..."
fn reformat_error_message(error_msg: &str) -> Option<String> {
    // Extract position from "Error type: line X column Y: message (line X column Y)"
    let colon_pos = error_msg.find(": line ")?;
    let after_type = &error_msg[colon_pos + 2..];

    let msg_start = after_type.find(": ")?;
    let position = &after_type[..msg_start];
    let rest = &after_type[msg_start + 2..];

    // Remove redundant position suffix " (line X column Y)"
    let message = rest
        .rfind(" (line ")
        .map_or(rest, |idx| &rest[..idx]);

    // Insert position before " while..." or " (if..." clause, or append at end
    if let Some(while_pos) = message.find(" while ") {
        let before = &message[..while_pos];
        let after = &message[while_pos + 1..];
        return Some(format!("{} at {}, {}", before, position, after));
    }

    if let Some(if_pos) = message.find(" (if ") {
        let before = &message[..if_pos];
        let after = &message[if_pos + 1..];
        return Some(format!("{} at {}, {}", before, position, after));
    }

    Some(format!("{} at {}", message, position))
}
