mod utils;
use std::error::Error;
use std::path::Path;
use crate::utils::{Blob, Object};

fn hash_file(path: &str) -> Result<(), Box<dyn Error>> {
    let mut blob = Blob::from_file(&path)?;
    let object = Object::from_blob(&mut blob);
    object.save(&Path::new("."))?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>>{
    std::env::set_current_dir("/tmp/test")?;
    println!("Hello, world!");
    hash_file("abc")
}
