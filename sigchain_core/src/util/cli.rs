use super::Result;

use std::io::{stderr, stdin, Read, Write};

pub fn prompt(msg: &str) -> Result<bool> {
    stderr().write(msg.as_bytes())?;
    stderr().write(b" [y/N]\r\n")?;

    let mut answer = vec![0u8];
    stdin().read(&mut answer)?;
    Ok(answer[0] == b'y')
}
