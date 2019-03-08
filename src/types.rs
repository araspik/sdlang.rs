use crate::parse;
use crate::{grammar, grammar::Rule};
use crate::{Error, Result};

use itertools::Itertools;

use std::fmt;
use std::iter;
use std::str::FromStr;
pub use std::time::Duration;

/// `chrono`'s timezone-aware date-time struct.
pub type DateTime = chrono::DateTime<chrono::FixedOffset>;
/// `chrono`'s timezone-naive date struct.
pub type Date = chrono::NaiveDate;

/// The value type encasing all possible SDLang value types.
///
/// This covers every single SDLang value there is.
/// It forms a shell around any of them, allowing them to stored and used
/// easily.
///
/// It implements `FromStr` to allow direct parsing, as well as `From` for all
/// its subtypes (except `Null`).
#[derive(PartialEq, Clone)]
pub enum Value {
    /// Text types. Both normal and raw strings come under this.
    String(String),
    /// Base64 binary data, in the form of a series of bytes.
    Base64(Vec<u8>),
    /// Date. Not timezone-aware.
    Date(Date),
    /// Date and time, timezone-aware.
    DateTime(DateTime),
    /// Durations of time.
    Duration(Duration),
    /// Integers.
    Number(i128),
    /// Decimals (floating-point).
    Decimal(f64),
    /// Boolean values.
    Boolean(bool),
    /// Null.
    Null,
}

impl fmt::Display for Value {
    /// Displays the value in a human-readable format.
    ///
    /// The type of the result is (should be) always distinguishable based on
    /// the format used.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Value::String(text) => write!(f, "\"{}\"", text),
            Value::Base64(data) => write!(f, "{:x?}", data),
            Value::Date(date) => write!(f, "{}", date),
            Value::DateTime(dtime) => write!(f, "{}", dtime),
            Value::Duration(dur) => write!(f, "{:#?}", dur),
            Value::Number(num) => write!(f, "{}", num),
            Value::Decimal(dec) => write!(f, "{}", dec),
            Value::Boolean(val) => write!(f, "{}", val),
            Value::Null => write!(f, "null"),
        }
    }
}

impl fmt::Debug for Value {
    /// Formats the same way as `fmt::Display`.
    /// This is done so that excessive output is not shown in any debug output,
    /// since no information is lost in using the typical format.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}

impl FromStr for Value {
    type Err = Error;

    /// Parses the text for a `Value`, returning a parse error on failure.
    fn from_str(s: &str) -> Result<Self> {
        grammar::parse(Rule::value, s).and_then(parse::value)
    }
}

impl From<String> for Value {
    /// Creates a `Value::String` from the given string.
    fn from(v: String) -> Self {
        Value::String(v)
    }
}

impl From<&str> for Value {
    /// Creates a `Value::String` from the given string, allocating.
    fn from(v: &str) -> Self {
        Value::String(v.to_string())
    }
}

impl From<Vec<u8>> for Value {
    /// Creates a `Value::Base64` from the given data.
    fn from(v: Vec<u8>) -> Self {
        Value::Base64(v)
    }
}

impl From<Date> for Value {
    /// Creates a `Value::Date` from the given date.
    fn from(v: Date) -> Self {
        Value::Date(v)
    }
}

impl From<DateTime> for Value {
    /// Creates a `Value::DateTime` from the given date and time.
    fn from(v: DateTime) -> Self {
        Value::DateTime(v)
    }
}

impl From<Duration> for Value {
    /// Creates a `Value::Duration` from the given duration.
    fn from(v: Duration) -> Self {
        Value::Duration(v)
    }
}

impl From<i128> for Value {
    /// Creates a `Value::Number` from the given integer.
    fn from(v: i128) -> Self {
        Value::Number(v)
    }
}

impl From<f64> for Value {
    /// Creates a `Value::Decimal` from the given decimal.
    fn from(v: f64) -> Self {
        Value::Decimal(v)
    }
}

impl From<bool> for Value {
    /// Creates a `Value::Boolean` from the given `bool`.
    fn from(v: bool) -> Self {
        Value::Boolean(v)
    }
}

/// A value with an associated name, forming a single attribute.
///
/// Multiple attributes may be placed on tags.
///
/// Conversion functions are providing for parsing and for converting to and
/// from a `(String, Value)` tuple (useful for collecting a set of attributes
/// into a hash map).
#[derive(PartialEq, Clone)]
pub struct Attribute {
    /// The name of the attribute.
    pub name: String,
    /// The associated value.
    pub value: Value,
}

impl Attribute {
    /// Creates a new Attribute.
    pub fn new(name: String, value: Value) -> Self {
        Attribute { name, value }
    }
}

impl fmt::Display for Attribute {
    /// Formats as `<name>: <value>`.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.value)
    }
}

impl fmt::Debug for Attribute {
    /// Formats the same way as `fmt::Display`. Useful for non-excessive debug
    /// output.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}

impl FromStr for Attribute {
    type Err = Error;

    /// Parses the text for an `Attribute`, returning a parse error on failure.
    fn from_str(s: &str) -> Result<Self> {
        grammar::parse(Rule::attribute, s).and_then(parse::attribute)
    }
}

impl From<(String, Value)> for Attribute {
    /// Converts from a key-value tuple.
    ///
    /// Useful when converting from an iterator of key-value pairs (which may
    /// originate from `HashMap::iter()`).
    fn from((k, v): (String, Value)) -> Self {
        Attribute { name: k, value: v }
    }
}

impl From<Attribute> for (String, Value) {
    /// Converts into a key-value tuple.
    ///
    /// Useful when converting into an iterator of key-value pairs (which may
    /// be collected into a `HashMap`).
    fn from(attr: Attribute) -> Self {
        (attr.name, attr.value)
    }
}

/// A series of values and attributes, with a name (and namespace), and
/// optionally a subtree of tags.
///
/// All data in SDLang is stored through these.
#[derive(Debug, PartialEq, Clone)]
pub struct Tag {
    /// The namespace (if any) of the tag.
    pub namespace: Option<String>,
    /// The name of the tag. `""` if none was given.
    pub name: String,
    /// A list of values.
    pub values: Vec<Value>,
    /// A list of attributes.
    pub attrs: Vec<Attribute>,
    /// A list of subtags (empty vector if none exist).
    pub tags: Vec<Tag>,
}

impl Tag {
    /// Creates a new tag.
    ///
    /// Note that no allocation is performed. Everything is initialised to a
    /// default empty state.
    pub fn new(name: String) -> Self {
        Tag {
            namespace: None,
            name,
            values: Vec::new(),
            attrs: Vec::new(),
            tags: Vec::new(),
        }
    }

    /// Sets the namespace.
    pub fn namespace(mut self, namespace: String) -> Self {
        self.namespace = Some(namespace);
        self
    }

    /// Sets the namespace from an `Option`.
    ///
    /// `None` unsets the namespace, and `Some(name)` sets the namespace to
    /// `name`.
    pub fn namespace_opt(mut self, namespace: Option<String>) -> Self {
        self.namespace = namespace;
        self
    }

    /// Sets the value list.
    pub fn values<I>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = Value>,
    {
        self.values.extend(values);
        self
    }

    /// Sets the attribute list from a list of Attributes.
    pub fn attrs<I>(mut self, attrs: I) -> Self
    where
        I: IntoIterator<Item = Attribute>,
    {
        self.attrs.extend(attrs);
        self
    }

    /// Sets the child tags.
    pub fn tags<I>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = Tag>,
    {
        self.tags.extend(tags);
        self
    }

    /// Finds the given attribute by name.
    pub fn attr<'a, 'b>(&'a self, name: &'b str) -> Option<&'a Attribute> {
        self.attrs.iter().find(|a| a.name == name)
    }

    /// Finds the given attribute by name, returning a mutable reference.
    pub fn attr_mut<'a, 'b>(
        &'a mut self,
        name: &'b str,
    ) -> Option<&'a mut Attribute> {
        self.attrs.iter_mut().find(|a| a.name == name)
    }

    /// Finds the given subtag by name.
    pub fn tag<'a, 'b>(&'a self, name: &'b str) -> Option<&'a Tag> {
        self.tags.iter().find(|t| t.name == name)
    }

    /// Finds the given tag by name, returning a mutable reference.
    pub fn tag_mut<'a, 'b>(&'a mut self, name: &'b str) -> Option<&'a mut Tag> {
        self.tags.iter_mut().find(|t| t.name == name)
    }
}

impl fmt::Display for Tag {
    /// Returns a human-readable representation of the tag.
    /// The format is roughly this:
    /// ```text
    /// tag "<namespace>:<name>": <values>, <attributes>
    /// * <subtag_1>
    ///   * <subtag_1_subtag>
    ///   * ...
    /// * <subtag_2>
    /// * ...
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "tag \"{}\": {}{}",
            self.namespace
                .iter()
                .chain(iter::once(&self.name))
                .format(":"),
            self.values
                .iter()
                .map(|v| v.to_string())
                .chain(self.attrs.iter().map(|a| format!("[{}]", a)))
                .format(", "),
            self.tags.iter().format_with("", |t, f| f(&format_args!(
                "\n* {}",
                t.to_string().replace('\n', "\n  ")
            ))),
        )
    }
}

impl FromStr for Tag {
    type Err = Error;

    /// Parses the text for a `Tag`, returning a parse error on failure.
    fn from_str(s: &str) -> Result<Self> {
        grammar::parse(Rule::tag, s).and_then(parse::tag)
    }
}
