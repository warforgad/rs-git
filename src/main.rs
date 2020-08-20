mod utils;
use std::error::Error;
use std::path::Path;
use crate::utils::{Blob, save_object};

fn hash_file(path: &str) -> Result<(), Box<dyn Error>> {
    let blob = Blob::from_file(&path)?;
    save_object(&Path::new("."), &blob)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>>{
    std::env::set_current_dir("/tmp/test")?;
    println!("Hello, world!");
    hash_file("abc")
}
