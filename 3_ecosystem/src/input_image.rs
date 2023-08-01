use std::path::PathBuf;

use async_compat::CompatExt;
use auto_enums::auto_enum;
use isahc::{
    http::{StatusCode, Uri},
    HttpClient,
};
use thiserror::Error;
use tokio::io::AsyncRead;
use url::Url;

#[derive(Debug, Clone)]
pub enum InputImage {
    Path(PathBuf),
    Uri(Uri),
}

#[derive(Debug, Error)]
pub enum InputImageError {
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("network error")]
    Net(#[from] isahc::Error),
    #[error("failed request")]
    FailedRequest(StatusCode),
}

pub type ClientResult<'a> = Result<&'a HttpClient, isahc::Error>;

impl InputImage {
    // using a function for the client so that one never gets initialized when there aren't any uri arguments
    #[auto_enum(tokio1::AsyncRead)]
    pub async fn open<'a, F: FnOnce() -> ClientResult<'a>>(
        &self,
        client_getter: F,
    ) -> Result<impl AsyncRead, InputImageError> {
        Ok(
            #[nested]
            match self {
                InputImage::Path(path) => tokio::fs::File::open(path).await?,
                InputImage::Uri(uri) => {
                    let client = client_getter()?;
                    let response = client.get_async(uri).await?;
                    let status = response.status();
                    if !status.is_success() {
                        return Err(InputImageError::FailedRequest(status));
                    }
                    response.into_body().compat()
                }
            },
        )
    }

    pub fn file_name(&self) -> Option<&str> {
        match self {
            InputImage::Path(path) => path.file_name().and_then(|file_name| file_name.to_str()),
            InputImage::Uri(uri) => uri
                .path()
                .strip_prefix('/')
                .map(|rest| rest.split('/'))
                .and_then(|segments| segments.last()),
        }
    }
}

impl From<String> for InputImage {
    fn from(value: String) -> Self {
        // I'm using `url::Url` to validate a full url here
        // since `http::Uri` doesn't appear to have any built-in functionality like that
        if value.parse::<Url>().is_ok() {
            if let Ok(uri) = value.parse::<Uri>() {
                return InputImage::Uri(uri);
            }
        }
        InputImage::Path(value.into())
    }
}
