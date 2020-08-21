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
    let path = git_dir.join(".git/objects").join(generate_object_path(data.as_slice()).unwrap());
    return save_into(data.as_slice(), &path);
}

pub trait Serializable
{
    fn serialize(&self) -> Vec<u8>;
}

pub struct Blob {
    data: Vec<u8>,
}

impl Blob {
    pub fn from_file(path: &str) -> Result<Blob> {
        Ok(Blob { data: std::fs::read(path)? })
    }

    pub fn from_data(raw_data: &[u8]) -> Blob {
        Blob { data: Vec::from(raw_data)}
    }
}

impl Serializable for Blob {
    fn serialize(&self) -> Vec<u8> {
        let mut serialized_data = Vec::from(format!("blob {}\x00", self.data.len()).as_bytes());
        serialized_data.extend(&self.data);
        return serialized_data
    }
}

struct TreeEntry {
    mode: u32,
    name: String,
    data: Box<dyn Serializable>,
}


impl Serializable for TreeEntry {
    fn serialize(&self) -> Vec<u8> {
        let mut serialized_data = Vec::from(format!("{} {}\x00", self.mode, &self.name));
        serialized_data.extend(hash_data(self.data.serialize().as_slice()).iter().cloned());
        return serialized_data;
    }
}

pub struct Tree {
    entries: Vec<TreeEntry>
}

impl Serializable for Tree {
    fn serialize(&self) -> Vec<u8> {
        let mut serialized_entries = Vec::new();
        for entry in &self.entries {
            serialized_entries.append(entry.serialize().as_mut())
        }

        let mut serialized_data = Vec::from(format!("tree {}\x00", serialized_entries.len()));
        serialized_data.append(serialized_entries.as_mut());

        return serialized_data
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

    #[test]
    fn test_save_blob() {
        let tempdir = setup().unwrap();
        let data = vec!(1,2,3);
        let blob = Blob::from_data(data.as_slice());
        let serialized = blob.serialize();
        save_object(tempdir.path(), &blob).unwrap();
        assert_stored_in_path(serialized.as_slice(), &generate_object_path(serialized.as_slice()).unwrap(), tempdir.path());
    }

    #[test]
    fn test_tree_serialization() {
        let blob1 = Blob::from_data(b"123");
        let blob2 = Blob::from_data(b"second");

        let mut serialized_blobs = Vec::new();
        serialized_blobs.extend_from_slice(b"4445 Blob1\x00");
        serialized_blobs.extend_from_slice(&hash_data(blob1.serialize().as_slice()));
        serialized_blobs.extend_from_slice(b"12 second\x00");
        serialized_blobs.extend_from_slice(&hash_data(blob2.serialize().as_slice()));

        let mut expected = Vec::from(format!("tree {}\x00", serialized_blobs.len()));
        expected.append(serialized_blobs.as_mut());

        let tree = Tree {
            entries: vec!(
                TreeEntry{mode: 4445, name: String::from("Blob1"), data: Box::new(blob1)},
                TreeEntry{mode: 12, name: String::from("second"), data: Box::new(blob2)}
            )
        };

        assert_eq!(expected, tree.serialize());
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

