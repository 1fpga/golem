use thiserror::Error;

/// Errors that can occur when reading a BK2 header file.
#[derive(Error, Debug)]
pub enum Bk2HeaderError {}

pub struct Bk2Header {
    movie_version: String,
    version: String,
    rerecord_count: usize,
    author: String,
    platform: String,
    game_name: String,
    sha1: String,
    core: String,
}

impl TryFrom<String> for Bk2Header {
    type Error = Bk2HeaderError;

    fn try_from(header: String) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl Bk2Header {}
