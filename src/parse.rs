use chrono::NaiveDateTime;
use grok::{Grok, Pattern};
use grok;

#[derive(Debug)]
pub struct LogLine {
    pub timestamp: Option<NaiveDateTime>,
    pub level: Option<Level>,
    pub message: Option<String>,
}

#[derive(Debug)]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Level {
    fn from_str(input: &str) -> Self {
        let upper = input.to_uppercase();
        match upper.as_ref() {
            "TRACE" => Level::Trace,
            "DEBUG" => Level::Debug,
            "INFO" => Level::Info,
            "WARN" => Level::Warn,
            "ERROR" => Level::Error,
            _ => panic!(),
        }
    }
}

#[derive(Debug)]
pub enum ParserError {
    GrokError(grok::Error),
}

impl From<grok::Error> for ParserError {
    fn from(error: grok::Error) -> Self {
        ParserError::GrokError(error)
    }
}

pub struct Parser {
    pattern: Pattern,
}

impl Parser {
    pub fn new(pattern: &str) -> Result<Self, ParserError> {
        let mut grok = Grok::default();
        let compiled_pattern = grok.compile(pattern, false)?;
        Ok(Parser {
            pattern: compiled_pattern,
        })
    }
    pub fn parse(&self, input: &str) -> Result<Option<LogLine>, ParserError> {
        Ok(match self.pattern.match_against(input) {
            Some(m) => Some(LogLine {
                timestamp: Some(
                    NaiveDateTime::parse_from_str(
                        m.get("timestamp").expect("No timestamp!"),
                        "%Y-%m-%dT%H:%M:%S,%f",
                    ).expect("could not parse date"),
                ),
                level: Some(Level::from_str(m.get("level").expect("no level!"))),
                message: Some(String::from(m.get("message").expect("no message!"))),
            }),
            None => None,
        })
    }
}
