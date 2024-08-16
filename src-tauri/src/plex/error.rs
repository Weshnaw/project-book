use derive_more::{Display, Error, From};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From, Error, Display)]
pub enum Error {
    InvalidHeader(reqwest::header::InvalidHeaderValue),
    RequestFailed(reqwest::Error),
    InvalidJson(serde_json::Error),
    WaitingOnPin,
    NoServerSelected,
    InvalidSeverName,
    NotAuthenticated,
    NoResourcesFound,
    NoLibrariesFound,
    InvalidLibraryName,
    NoValidConnections,
    MediaContainerNotFound,
    LibraryDirectoryNotFound,
}
