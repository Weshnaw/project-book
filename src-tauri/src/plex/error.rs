use derive_more::{Display, Error, From};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From, Error, Display)]
pub enum Error {
    InvalidHeader(reqwest::header::InvalidHeaderValue),
    RequestFailed(reqwest::Error),
    InvalidJson(serde_json::Error),
    WaitingOnPin,
    NoServerSelected,
    NoLibrarySelected,
    InvalidSeverName,
    NotAuthenticated,
    NoResourcesFound,
    NoAlbumFound,
    InvalidLibraryName,
    NoValidConnections,
    MediaContainerNotFound,
    LibraryDirectoryNotFound,
    LibraryMetadataNotFound,
    NoAlbumsFound,
    NoLibrariesFound,
    NoThumbnailFound,
}
