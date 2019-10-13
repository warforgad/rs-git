extern crate hex;

use std::path::Path;
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

    #[allow(dead_code)]
    pub fn from_data(data: &[u8]) -> Object {
        Object {
            data: data.to_vec(),
            digest: hash_data(&data)
        }
    }

    pub fn save(&self, dir: &Path) -> Result<()> {
        let hash = hex::encode(&self.digest);
        let dirname = dir.join(".git/objects").join(&hash[..2]);
        let filename = &hash[2..];
        if ! (dirname.is_dir()) {
            std::fs::create_dir(&dirname)?;
        }
        std::fs::write(dirname.join(&filename), &self.data)?;
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
        Ok(Blob { data: std::fs::read(path)? })
    }
}

fn hash_data(data: &[u8]) -> [u8;20] {
    let mut digest: [u8; 20] = [0; 20];
    let mut hasher = Sha1::new();
    hasher.input(&data);
    hasher.result(&mut digest);
    return digest;
}

impl Hashable for Blob {
    fn hash(&self) -> [u8; 20] {
        hash_data(&self.data)
    }
}

#[cfg(test)]
mod tests {
    extern crate tempdir;
    use super::*;
    use self::tempdir::TempDir;

    fn setup() -> Result<TempDir> {
        let tempdir = TempDir::new("")?;
        std::fs::create_dir_all(tempdir.path().join(".git/objects"))?;
        return Ok(tempdir)
    }

    fn assert_stored_in_path(data: &[u8], path: &Path, dir: &Path) {
        let store_path = dir.join(".git/objects").join(path);
        assert_eq!(data,
                   std::fs::read(store_path).unwrap().as_slice());
    }

    fn assert_stored_hash(data: &[u8], hash: &str, dir: &Path) {
        assert_stored_in_path(&data, &Path::new(&hash[..2]).join(&hash[2..]), dir)
    }

    fn assert_stored(data: &[u8], dir: &Path) {
        assert_stored_hash(&data, hex::encode(hash_data(&data)).as_str(), dir);
    }

    #[test]
    fn test_save() {
        let tempdir = setup().unwrap();
        let data: Vec<u8> = vec!(1, 2, 3);
        let object = Object::from_data(data.as_slice());
        object.save(tempdir.path()).unwrap();
        assert_stored(data.as_slice(), tempdir.path());
    }

    #[test]
    fn test_same_store_dir() {
        let tempdir = setup().unwrap();
        let data: Vec<u8> = vec!(1, 2, 3);
        let mut other_data = data.clone();
        other_data.reverse();
        let other_digest = [0x70; 20];
        let object = Object::from_data(data.as_slice());
        object.save(tempdir.path()).unwrap();
        Object { data: other_data.clone(), digest: other_digest }.save(tempdir.path()).unwrap();
        assert_stored(data.as_slice(), tempdir.path());
        assert_stored_hash(other_data.as_slice(), hex::encode(other_digest).as_str(), tempdir.path())
    }
}

