use std::fmt::{write, Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct HeaderParseError;

impl Display for HeaderParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "HeaderParseError")
    }
}

#[derive(Debug)]
pub struct Header {
    key: String,
    value: String,
}

impl Header {
    pub fn new(key: String, value: String) -> Self {
        Self { key, value }
    }
}

impl FromStr for Header {
    type Err = HeaderParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut spl = s.splitn(2, ": ");
        let key = spl.next().ok_or(Self::Err{})?;
        let value = spl.next().ok_or(Self::Err{})?;
        Ok(Self::new(key.to_string(), value.to_string()))
    }
}

#[derive(Debug)]
pub struct Headers(Vec<Header>);

impl Headers {
    pub fn add(&mut self, key: String, value: String) {
        self.0.push(Header::new(key, value));
    }

    pub fn push(&mut self, header: Header) {
        self.0.push(header);
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self
            .0
            .iter()
            .find(|x| x.key.to_lowercase() == key.to_lowercase())
            .map(|x| x.value.as_str())
    }
}

impl Into<String> for Headers {
    fn into(self) -> String {
        self.0
            .iter()
            .map(|header| format!("{}: {}", header.key, header.value))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl From<Vec<Header>> for Headers {
    fn from(value: Vec<Header>) -> Self {
        Self(value)
    }
}

impl From<Vec<(String, String)>> for Headers {
    fn from(value: Vec<(String, String)>) -> Self {
        Headers(
            value
                .into_iter()
                .map(|(key, value)| Header::new(key, value))
                .collect(),
        )
    }
}
