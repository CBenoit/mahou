use owo_colors::OwoColorize;
use std::{fmt, result::Result as StdResult};
use thiserror::Error;

pub mod nibl;
pub use nibl::Nibl;

/////////////////////////////////////////////////
//                    Error                    //
/////////////////////////////////////////////////
#[derive(Debug, Error)]
pub enum Error {
    #[error("Request failed: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("The {api} API returned and error: {message}")]
    APIError { api: &'static str, message: String },
}

type Result<T> = std::result::Result<T, Error>;

//////////////////////////////////////////////////
//                    Finder                    //
//////////////////////////////////////////////////
pub trait Finder {
    fn find(&self, query: &Query) -> Result<Vec<Entry>>;
}

/////////////////////////////////////////////////////////
//                    EpisodeNumber                    //
/////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy, Default, PartialEq, Hash)]
pub enum EpisodeNumber {
    #[default]
    All,
    Latest,
    Number(i32),
}

impl std::str::FromStr for EpisodeNumber {
    type Err = String;

    fn from_str(s: &str) -> StdResult<Self, Self::Err> {
        match s {
            "latest" => Ok(Self::Latest),
            _ => s
                .parse::<i32>()
                .map(Self::Number)
                .map_err(|_| format!("Invalid episode number {}", s)),
        }
    }
}

impl fmt::Display for EpisodeNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> StdResult<(), fmt::Error> {
        match self {
            Self::Number(n) => write!(f, "{}", n),
            Self::Latest => write!(f, "latest"),
            Self::All => write!(f, "all"),
        }
    }
}

/////////////////////////////////////////////////
//                    Query                    //
/////////////////////////////////////////////////
#[derive(Debug, Default, Clone, PartialEq, Hash)]
pub struct Query {
    pub search: String,
    pub resolution: String,
    pub episode: EpisodeNumber,
}

impl Query {
    pub fn new(search: String, resolution: String, episode: EpisodeNumber) -> Self {
        Self { search, resolution, episode }
    }

    pub fn find<F: Finder>(&self, finder: &F) -> Result<Vec<Entry>> {
        finder.find(self)
    }

    pub fn find_many(&self, finders: &[Box<dyn Finder>]) -> Result<Vec<Entry>> {
        // TODO: figure out how to do this with an iterator chain
        let mut entries = Vec::new();
        for finder in finders {
            entries.append(&mut finder.find(self)?);
        }
        Ok(entries)
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Entry {
    pub package_number: i32,
    pub bot_id: i64,
    pub bot_name: String,
    pub name: String,
    pub size: String,
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> StdResult<(), fmt::Error> {
        write!(
            f,
            "{} [{}] ({})",
            self.name,
            self.bot_name.yellow(),
            self.size,
        )
    }
}
