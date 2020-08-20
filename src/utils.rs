extern crate hex;

use std::path::{Path, PathBuf};
use std::error::Error;
use crypto::sha1::Sha1;
use crypto::digest::Digest;

type Result<T> = std::result::Result<T, Box<Error>>;

fn save_into(data: &[u8], path: &Path) -> Result<()> {
    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::write(path, &data)?;
    Ok(())
}

fn generate_object_path(data: &[u8]) -> Result<PathBuf> {
    let hash = hex::encode(&hash_data(&data));
    let mut path = PathBuf::from(&hash[0..2]);
    path.push(&hash[2..]);
    Ok(path)
}

pub fn save_object(git_dir: &Path, object: &impl Serializable) -> Result<()> {
    let data = object.serialize();
    let path = git_dir.join(".git/objects").join(generate_object_path(&data).unwrap());
    let mut full_data = Vec::from(format!("{} {}\x00", object.get_name(), data.len()).as_bytes());
    full_data.extend_from_slice(&data);
    return save_into(full_data.as_slice(), &path);
}

pub trait Serializable
{
    fn serialize(&self) -> &[u8];
    fn get_name(&self) -> String;
}

pub struct Blob {
    data: Vec<u8>,
}

impl Blob {
    pub fn from_file(path: &str) -> Result<Blob> {
        Ok(Blob { data: std::fs::read(path)? })
    }

    pub fn from_data(raw_data: &[u8]) -> Result<Blob> {Ok(Blob { data: Vec::from(raw_data)}) }
}

impl Serializable for Blob {
    fn serialize(&self) -> &[u8] {
        self.data.as_slice()
    }
    fn get_name(&self) -> String { String::from("blob") }
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

    #[test]
    fn test_save_blob() {
        let tempdir = setup().unwrap();
        let data = vec!(1,2,3);
        let blob = Blob::from_data(data.as_slice()).unwrap();
        save_object(tempdir.path(), &blob).unwrap();
        let mut expected = Vec::from(format!("{} {}\x00", blob.get_name(), data.len()));
        expected.extend_from_slice(&data);
        assert_stored_in_path(expected.as_slice(), &generate_object_path(&data).unwrap(), tempdir.path());
    }

/*    #[test]
    fn test_same_store_dir() {
        let tempdir = setup().unwrap();
        let data: Vec<u8> = vec!(1, 2, 3);
        let other_data: Vec<u8> = vec!(3, 2, 1);
        Object::save_into(&data, &tempdir.path().join(".git/objects/00/0")).unwrap();
        Object::save_into(&other_data, &tempdir.path().join(".git/objects/00/1")).unwrap();
        assert_stored_hash(data.as_slice(), "000", tempdir.path());
        assert_stored_hash(other_data.as_slice(), "001", tempdir.path());
    }
*/
}

