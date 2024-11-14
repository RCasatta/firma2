use std::io::Read;

pub use bitcoin;
pub use clap;
pub use error::Error;
pub use seed::Seed;
pub use serde_json;

mod error;
mod seed;

pub mod addresses;
pub mod derive;
pub mod sign;

/// Read standard input as string, trimming new lines
pub fn read_stdin() -> Result<String, Error> {
    let mut stdin = std::io::stdin().lock();
    let mut buffer = vec![];
    stdin.read_to_end(&mut buffer)?;
    let s = std::str::from_utf8(&buffer)?;

    Ok(s.chars().filter(|c| *c != '\n').collect())
}

/// Read standard input as string, trimming new lines
pub fn read_stdin_seed() -> Result<Seed, Error> {
    let s = read_stdin()?;
    Ok(s.parse()?)
}

fn debug_to_string<D: std::fmt::Debug>(d: D) -> String {
    format!("{d:?}")
}
