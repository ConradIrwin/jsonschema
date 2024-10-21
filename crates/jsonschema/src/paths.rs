//! Facilities for working with paths within schemas or validated instances.
use std::{fmt, sync::Arc};

use referencing::List;

use crate::keywords::Keyword;

#[deprecated(
    since = "0.23.0",
    note = "Use `jsonschema::paths::LocationSegment` instead. This alias will be removed in a future release."
)]
pub type PathChunkRef<'a> = LocationSegment<'a>;
#[deprecated(
    since = "0.23.0",
    note = "Use `jsonschema::paths::LazyLocation` instead. This alias will be removed in a future release."
)]
pub type JsonPointerNode<'a, 'b> = LazyLocation<'a, 'b>;

/// A location segment.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LocationSegment<'a> {
    /// Property name within a JSON object.
    Property(&'a str),
    /// JSON Schema keyword.
    Index(usize),
}

impl<'a> fmt::Display for LocationSegment<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocationSegment::Property(property) => f.write_str(property),
            LocationSegment::Index(idx) => f.write_str(itoa::Buffer::new().format(*idx)),
        }
    }
}

/// A lazily constructed location within a JSON instance.
///
/// [`LazyLocation`] builds a path incrementally during JSON Schema validation without allocating
/// memory until required by storing each segment on the stack.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LazyLocation<'a, 'b> {
    pub(crate) segment: LocationSegment<'a>,
    pub(crate) parent: Option<&'b LazyLocation<'b, 'a>>,
}

impl Default for LazyLocation<'_, '_> {
    fn default() -> Self {
        LazyLocation::new()
    }
}

impl<'a, 'b> LazyLocation<'a, 'b> {
    /// Create a root node of a JSON pointer.
    pub const fn new() -> Self {
        LazyLocation {
            // The value does not matter, it will never be used
            segment: LocationSegment::Index(0),
            parent: None,
        }
    }

    /// Push a new segment to the JSON pointer.
    #[inline]
    pub fn push(&'a self, segment: impl Into<LocationSegment<'a>>) -> Self {
        LazyLocation {
            segment: segment.into(),
            parent: Some(self),
        }
    }
}

impl<'a> From<List<LocationSegment<'a>>> for Location {
    fn from(value: List<LocationSegment<'a>>) -> Self {
        let mut capacity = 0;
        let mut string_capacity = 0;
        for segment in value.iter() {
            capacity += 1;
            string_capacity += match segment {
                LocationSegment::Property(property) => property.len() + 1,
                LocationSegment::Index(idx) => idx.checked_ilog10().unwrap_or(0) as usize + 2,
            };
        }

        let mut buffer = String::with_capacity(string_capacity);

        let mut segments = Vec::with_capacity(capacity);

        for segment in value.iter() {
            segments.push(segment);
        }

        for segment in segments.iter().rev() {
            buffer.push('/');
            match segment {
                LocationSegment::Property(property) => {
                    write_escaped_str(&mut buffer, property);
                }
                LocationSegment::Index(idx) => {
                    let mut itoa_buffer = itoa::Buffer::new();
                    buffer.push_str(itoa_buffer.format(*idx));
                }
            }
        }

        Location(Arc::new(buffer))
    }
}

impl<'a> From<&'a LazyLocation<'_, '_>> for Location {
    fn from(value: &'a LazyLocation<'_, '_>) -> Self {
        let mut capacity = 0;
        let mut string_capacity = 0;
        let mut head = value;

        while let Some(next) = head.parent {
            capacity += 1;
            string_capacity += match head.segment {
                LocationSegment::Property(property) => property.len() + 1,
                LocationSegment::Index(idx) => idx.checked_ilog10().unwrap_or(0) as usize + 2,
            };
            head = next;
        }

        let mut buffer = String::with_capacity(string_capacity);

        let mut segments = Vec::with_capacity(capacity);
        head = value;

        if head.parent.is_some() {
            segments.push(head.segment);
        }

        while let Some(next) = head.parent {
            head = next;
            if head.parent.is_some() {
                segments.push(head.segment);
            }
        }

        for segment in segments.iter().rev() {
            buffer.push('/');
            match segment {
                LocationSegment::Property(property) => {
                    write_escaped_str(&mut buffer, property);
                }
                LocationSegment::Index(idx) => {
                    let mut itoa_buffer = itoa::Buffer::new();
                    buffer.push_str(itoa_buffer.format(*idx));
                }
            }
        }

        Location(Arc::new(buffer))
    }
}

impl<'a> From<&'a Keyword> for LocationSegment<'a> {
    fn from(value: &'a Keyword) -> Self {
        match value {
            Keyword::Buildin(k) => LocationSegment::Property(k.as_str()),
            Keyword::Custom(s) => LocationSegment::Property(s),
        }
    }
}

impl<'a> From<&'a str> for LocationSegment<'a> {
    #[inline]
    fn from(value: &'a str) -> LocationSegment<'a> {
        LocationSegment::Property(value)
    }
}

impl<'a> From<&'a String> for LocationSegment<'a> {
    #[inline]
    fn from(value: &'a String) -> LocationSegment<'a> {
        LocationSegment::Property(value)
    }
}

impl From<usize> for LocationSegment<'_> {
    #[inline]
    fn from(value: usize) -> Self {
        LocationSegment::Index(value)
    }
}

/// A cheap to clone JSON pointer that represents location with a JSON value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location(Arc<String>);

impl Location {
    /// Create a new, empty `Location`.
    pub fn new() -> Self {
        Self(Arc::new(String::new()))
    }
    pub(crate) fn join<'a>(&self, segment: impl Into<LocationSegment<'a>>) -> Self {
        let parent = self.0.as_str();
        match segment.into() {
            LocationSegment::Property(property) => {
                let mut buffer = String::with_capacity(parent.len() + property.len() + 1);
                buffer.push_str(parent);
                buffer.push('/');
                write_escaped_str(&mut buffer, property);
                Self(Arc::new(buffer))
            }
            LocationSegment::Index(idx) => {
                let mut buffer = itoa::Buffer::new();
                let segment = buffer.format(idx);
                Self(Arc::new(format!("{parent}/{segment}")))
            }
        }
    }
    /// Get a string slice representing the location.
    pub fn as_str(&self) -> &str {
        &self.0
    }
    /// Get a byte slice representing the location.
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

fn write_escaped_str(buffer: &mut String, value: &str) {
    match value.find(['~', '/']) {
        Some(mut escape_idx) => {
            let mut remaining = value;

            // Loop through the string to replace `~` and `/`
            loop {
                let (before, after) = remaining.split_at(escape_idx);
                // Copy everything before the escape char
                buffer.push_str(before);

                // Append the appropriate escape sequence
                match after.as_bytes()[0] {
                    b'~' => buffer.push_str("~0"),
                    b'/' => buffer.push_str("~1"),
                    _ => unreachable!(),
                }

                // Move past the escaped character
                remaining = &after[1..];

                // Find the next `~` or `/` to continue escaping
                if let Some(next_escape_idx) = remaining.find(['~', '/']) {
                    escape_idx = next_escape_idx;
                } else {
                    // Append any remaining part of the string
                    buffer.push_str(remaining);
                    break;
                }
            }
        }
        None => {
            // If no escape characters are found, append the segment as is
            buffer.push_str(value);
        }
    };
}

impl Default for Location {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test]
    fn test_location_default() {
        let loc = Location::default();
        assert_eq!(loc.as_str(), "");
    }

    #[test]
    fn test_location_new() {
        let loc = Location::new();
        assert_eq!(loc.as_str(), "");
    }

    #[test]
    fn test_location_join_property() {
        let loc = Location::new();
        let loc = loc.join("property");
        assert_eq!(loc.as_str(), "/property");
    }

    #[test]
    fn test_location_join_index() {
        let loc = Location::new();
        let loc = loc.join(0);
        assert_eq!(loc.as_str(), "/0");
    }

    #[test]
    fn test_location_join_multiple() {
        let loc = Location::new();
        let loc = loc.join("property").join(0);
        assert_eq!(loc.as_str(), "/property/0");
    }

    #[test]
    fn test_as_bytes() {
        let loc = Location::new().join("test");
        assert_eq!(loc.as_bytes(), b"/test");
    }

    #[test]
    fn test_display_trait() {
        let loc = Location::new().join("property");
        assert_eq!(format!("{}", loc), "/property");
    }

    #[test_case("tilde~character", "/tilde~0character"; "escapes tilde")]
    #[test_case("slash/character", "/slash~1character"; "escapes slash")]
    #[test_case("combo~and/slash", "/combo~0and~1slash"; "escapes tilde and slash combined")]
    #[test_case("multiple~/escapes~", "/multiple~0~1escapes~0"; "multiple escapes")]
    #[test_case("first/segment", "/first~1segment"; "escapes slash in nested segment")]
    fn test_location_escaping(segment: &str, expected: &str) {
        let loc = Location::new().join(segment);
        assert_eq!(loc.as_str(), expected);
    }
}
