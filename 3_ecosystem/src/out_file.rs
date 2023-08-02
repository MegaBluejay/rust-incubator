use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use tokio::fs::{File, OpenOptions};
use tracing::{info, instrument};

#[instrument]
pub async fn create_out_file(
    out_dir: &Path,
    name: &str,
    extension: &str,
) -> Result<File, std::io::Error> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    for out_path in out_paths(out_dir, name, extension) {
        match options.open(&out_path).await {
            Ok(file) => {
                info!(?out_path);
                return Ok(file);
            }
            Err(err) => match err.kind() {
                ErrorKind::AlreadyExists => {}
                _ => return Err(err),
            },
        }
    }
    unreachable!()
}

struct OutNames<'a> {
    name: &'a str,
    extension: &'a str,
    i: u32,
}

impl<'a> Iterator for OutNames<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let res = if self.i == 0 {
            format!("{}.{}", self.name, self.extension)
        } else {
            format!("{}({}).{}", self.name, self.i, self.extension)
        };
        self.i += 1;
        Some(res)
    }
}

impl<'a> OutNames<'a> {
    fn new(name: &'a str, extension: &'a str) -> Self {
        Self {
            name,
            extension,
            i: 0,
        }
    }
}

fn out_paths<'a>(
    out_dir: &'a Path,
    name: &'a str,
    extension: &'a str,
) -> impl Iterator<Item = PathBuf> + 'a {
    OutNames::new(name, extension).map(|out_name| out_dir.join(out_name))
}
