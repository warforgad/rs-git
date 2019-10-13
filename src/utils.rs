extern crate hex;

use std::fs::File;
use std::path::Path;
use std::io::{Read, Write};
use std::error::Error;
use crypto::sha1::Sha1;
use crypto::digest::Digest;

type Result<T> = std::result::Result<T, Box<Error>>;

pub struct Object {
    digest: [u8; 20],
    data: Vec<u8>,
}

impl Object {
    pub fn from_blob(blob: &Blob) -> Object {
        Object {
            data: blob.data.clone(),
            digest: blob.hash()
        }
    }

    pub fn save(&self) -> Result<()> {
        let hash = hex::encode(&self.digest);
        let dirname = Path::new(".git/objects").join(&hash[..2]);
        let filename = &hash[2..];
        if ! (std::path::Path::new(&dirname).is_dir()) {
            std::fs::create_dir(&dirname)?;
        }
        let mut file = File::create(dirname.join(&filename))?;
        file.write_all(&self.data)?;
        Ok(())
    }
}

pub trait Hashable {
    fn hash(&self) -> [u8; 20];
}

pub struct Blob {
    data: Vec<u8>,
}

impl Blob {
    pub fn from_file(path: &str) -> Result<Blob> {
        let mut file = File::open(path)?;
        let mut data: Vec<u8> = Vec::new();
        file.read_to_end(&mut data)?;
        Ok(Blob { data })
    }
}

impl Hashable for Blob {
    fn hash(&self) -> [u8; 20] {
        let mut digest: [u8; 20] = [0; 20];
        let mut hasher = Sha1::new();
        hasher.input(&self.data);
        hasher.result(&mut digest);
        return digest;
    }
}

