use std::{fs::File, io::Read};

use serde::Deserialize;

pub fn read_ron_from_file<T: for<'a> Deserialize<'a>>(
    filepath: &str,
) -> Result<T, anyhow::Error> {
    let mut file = File::open(filepath)?;
    let mut file_content = String::new();
    file.read_to_string(&mut file_content)?;
    Ok(ron::from_str::<T>(&file_content)?)
}
