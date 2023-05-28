pub mod errors;
pub mod tl;
mod tl_iterator;
mod utils;

use errors::ParseError;
use tl::Definition;
use tl_iterator::TlIterator;

pub fn parse_tl_file(contents: &str) -> impl Iterator<Item = Result<Definition, ParseError>> {
    TlIterator::new(contents)
}