use derive_more::{Display, Error, From};

type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From, Error, Display)]
pub enum Error {}
