use std::{num::ParseIntError, sync::PoisonError};

use derive_more::{Display, Error, From};
use log::error;
use serde_json::json;
use tauri::ipc::InvokeError;

use crate::{plex, state};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From, Error, Display)]
pub enum Error {
    Plex(plex::Error),
    State(state::Error),
    Template(askama::Error),
    Tauri(tauri::Error),
    InvalidNumber(ParseIntError),
    FailedToLockState,
    NoChange,
}

impl From<Error> for InvokeError {
    fn from(val: Error) -> Self {
        error!("Command failed: {:#?}", val);
        Self(json!(format!("{{\"error\": \"{:?}\"}}", val)))
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_err: PoisonError<T>) -> Self {
        Self::FailedToLockState
    }
}
