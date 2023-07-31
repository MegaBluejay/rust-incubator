use std::path::PathBuf;

use auto_enums::auto_enum;
use bytes::Bytes;
use futures::{Stream, TryStreamExt};
use futures_enum::Stream;
use thiserror::Error;
use tokio::io::BufReader;
use tokio_util::io::ReaderStream;
use url::Url;

use super::global_client::CLIENT;

#[derive(Debug, Clone)]
pub enum InputImage {
    Path(PathBuf),
    Url(Url),
}

#[derive(Debug, Error)]
pub enum InputImageError {
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("network error")]
    Net(#[from] reqwest::Error),
}

impl InputImage {
    #[auto_enum(Stream)]
    pub async fn bytes_stream(
        self,
    ) -> Result<impl Stream<Item = Result<Bytes, InputImageError>>, InputImageError> {
        Ok(
            #[nested]
            match self {
                InputImage::Path(path) => {
                    let file = tokio::fs::File::open(path).await?;
                    ReaderStream::new(BufReader::new(file)).err_into()
                }
                InputImage::Url(url) => {
                    let response = CLIENT.get(url).send().await?.error_for_status()?;
                    response.bytes_stream().err_into()
                }
            },
        )
    }

    pub fn file_name(&self) -> Option<&str> {
        match self {
            InputImage::Path(path) => path.file_name().and_then(|file_name| file_name.to_str()),
            InputImage::Url(url) => url.path_segments().and_then(|segments| segments.last()),
        }
    }
}

impl From<String> for InputImage {
    fn from(value: String) -> Self {
        Url::parse(&value)
            .map(InputImage::Url)
            .unwrap_or_else(|_| InputImage::Path(value.into()))
    }
}
