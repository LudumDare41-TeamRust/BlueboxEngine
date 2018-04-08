use assets::LevelId;

#[derive(Debug, Copy, Clone)]
pub struct Error {

}

#[derive(Debug)]
pub enum LevelLoadError {
    Io(::std::io::Error),
    GameDataIsNotAFolder,
    InvalidLevelToLoad(LevelId),
    FileNameEncodingError,
    InvalidParentDirectory,
}

impl From<::std::io::Error> for LevelLoadError {
    fn from(e: ::std::io::Error) -> Self {
        LevelLoadError::Io(e)
    }
}