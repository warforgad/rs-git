extern crate hex;

use std::path::Path;
use std::error::Error;
use crypto::sha1::Sha1;
use crypto::digest::Digest;

type Result<T> = std::result::Result<T, Box<Error>>;

pub struct Object {
    data: Vec<u8>,
}

impl Object {
    #[allow(dead_code)]
    pub fn from_data(data: &[u8]) -> Object {
        Object {
            data: data.to_vec(),
        }
    }

    pub fn save_into(data: &[u8], path: &Path) -> Result<()> {
        std::fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(path, &data)?;
        Ok(())
    }

    pub fn save(&self, dir: &Path) -> Result<()> {
        let hash = hex::encode(&hash_data(&self.data));
        let path = dir.join(".git/objects").join(&hash[..2]).join(&hash[2..]);
        return Object::save_into(&self.data, &path);
    }
}

pub struct Blob {
    data: Vec<u8>,
}

impl Blob {
    pub fn from_file(path: &str) -> Result<Blob> {
        Ok(Blob { data: std::fs::read(path)? })
    }

    pub fn serialize(&self) -> Object {
        let mut data = format!("blob {}", self.data.len()).into_bytes();
        data.append(&mut self.data.clone());
        Object{data}
    }
}

fn hash_data(data: &[u8]) -> [u8;20] {
    let mut digest: [u8; 20] = [0; 20];
    let mut hasher = Sha1::new();
    hasher.input(&data);
    hasher.result(&mut digest);
    return digest;
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
        let other_data: Vec<u8> = vec!(3, 2, 1);
        Object::save_into(&data, &tempdir.path().join(".git/objects/00/0")).unwrap();
        Object::save_into(&other_data, &tempdir.path().join(".git/objects/00/1")).unwrap();
        assert_stored_hash(data.as_slice(), "000", tempdir.path());
        assert_stored_hash(other_data.as_slice(), "001", tempdir.path());
    }
}

