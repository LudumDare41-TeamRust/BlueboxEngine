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
    InvalidToml,
    TomlParseError(::toml::de::Error),
}

impl From<::std::io::Error> for LevelLoadError {
    fn from(e: ::std::io::Error) -> Self {
        LevelLoadError::Io(e)
    }
}

impl From<::toml::de::Error> for LevelLoadError {
    fn from(e: ::toml::de::Error) -> Self {
        LevelLoadError::TomlParseError(e)
    }
}