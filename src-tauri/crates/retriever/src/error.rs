use std::fmt;

#[derive(Debug)]
pub enum SearchError {
    Tantivy(tantivy::TantivyError),
    Io(std::io::Error),
}

impl fmt::Display for SearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SearchError::Tantivy(e) => write!(f, "search error: {e}"),
            SearchError::Io(e) => write!(f, "io error: {e}"),
        }
    }
}

impl std::error::Error for SearchError {}

impl From<tantivy::TantivyError> for SearchError {
    fn from(err: tantivy::TantivyError) -> Self {
        SearchError::Tantivy(err)
    }
}

impl From<std::io::Error> for SearchError {
    fn from(err: std::io::Error) -> Self {
        SearchError::Io(err)
    }
}

pub type Result<T> = std::result::Result<T, SearchError>;
