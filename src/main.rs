use std::{fs, io};

fn main() -> io::Result<()> {
    let code = fs::read_to_string("./code.exp")?;
    exp_lang::run(code);
    Ok(())
}
