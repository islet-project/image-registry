use std::{fs::File, io::Read, io::Write};

#[allow(dead_code)]
pub fn file_read(filename: &str) -> std::io::Result<Vec<u8>>
{
    let mut buf = Vec::new();
    File::open(filename)?.read_to_end(&mut buf)?;
    Ok(buf)
}

#[allow(dead_code)]
pub fn file_write(filename: &str, data: &[u8]) -> std::io::Result<()>
{
    File::create(filename)?.write_all(data)
}
