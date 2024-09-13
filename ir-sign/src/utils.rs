use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub fn file_read<T: AsRef<Path>>(filename: T) -> std::io::Result<Vec<u8>>
{
    let mut buf = Vec::new();
    File::open(filename)?.read_to_end(&mut buf)?;
    Ok(buf)
}

pub fn file_write<T: AsRef<Path>>(filename: T, data: &[u8]) -> std::io::Result<()>
{
    File::create(filename)?.write_all(data)
}

pub fn file_len<T: AsRef<Path>>(filename: T) -> std::io::Result<u64>
{
    Ok(std::fs::metadata(filename)?.len())
}
