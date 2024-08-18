use core::str;
use std::ffi::OsStr;
use std::path::Path;
use std::{fs, io};

use crate::http10::content_types::get_mime;

const TRYFILES: [&'static str; 2] = ["/index.html", "/index.htm"];

#[derive(Debug)]
pub enum FileError {
    ReadError(io::Error),
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct File {
    path: String,
    extension: Option<String>,
    mime_type: String,
    content: Vec<u8>,
    size: usize,
}

impl std::fmt::Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(str::from_utf8(&self.content).map_err(|_| std::fmt::Error)?)
    }
}

impl File {
    pub fn try_load(uri: String, base_dir: &str) -> Result<Self, FileError> {
        let path = Path::new(base_dir).join(&uri[1..]);
        if let Ok(exists) = path.try_exists() {
            if !exists {
                return Err(FileError::ReadError(io::ErrorKind::NotFound.into()));
            }
        } else {
            return Err(FileError::ReadError(io::ErrorKind::NotFound.into()));
        }
        if path.is_dir() {
            let try_files: Vec<Result<Self, FileError>> = TRYFILES
                .iter()
                .map(|file| {
                    Self::try_load(
                        Path::new(&uri).join(file).to_str().unwrap().to_string(),
                        base_dir,
                    )
                })
                .collect();
            if let Some(file) = try_files.into_iter().find_map(Result::ok) {
                return Ok(file);
            } else {
                return Err(FileError::ReadError(io::ErrorKind::NotFound.into()));
            }
        }
        let extension: Option<String> = path
            .extension()
            .and_then(OsStr::to_str)
            .map(|ext| ext.to_string());
        let mime_type = get_mime(extension.clone().unwrap_or("".to_string())).to_string();
        let content: Result<Vec<u8>, std::io::Error> = fs::read(&path);

        if content.is_ok() {
            let size = content.as_ref().unwrap().len();
            Ok(File {
                path: path.to_str().unwrap().to_string(),
                extension,
                content: content.unwrap(),
                mime_type,
                size,
            })
        } else {
            Err(FileError::ReadError(content.unwrap_err()))
        }
    }

    pub fn get_content(&self) -> Vec<u8> {
        self.content.clone()
    }

    pub fn get_mime(&self) -> String {
        self.mime_type.to_string()
    }

    pub fn get_size(&self) -> usize {
        self.size
    }
}
