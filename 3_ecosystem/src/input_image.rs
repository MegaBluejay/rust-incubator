use std::path::PathBuf;

use async_compat::CompatExt;
use auto_enums::auto_enum;
use isahc::{http::Uri, HttpClient};
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
    Net(#[from] isahc::error::Error),
}

impl InputImage {
    #[auto_enum(tokio1::AsyncRead)]
    pub async fn open<'a, F: Fn() -> &'a HttpClient>(
        self,
        client_getter: F,
    ) -> Result<impl AsyncRead, InputImageError> {
        Ok(
            #[nested]
            match self {
                InputImage::Path(path) => tokio::fs::File::open(path).await?,
                InputImage::Uri(uri) => {
                    let client = client_getter();
                    let response = client.get_async(uri).await?;
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
        if value.parse::<Url>().is_ok() {
            if let Ok(uri) = value.parse::<Uri>() {
                return InputImage::Uri(uri);
            }
        }
        InputImage::Path(value.into())
    }
}
