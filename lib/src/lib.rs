use std::io::Read;

pub use bitcoin;
pub use clap;
pub use error::Error;
use seed::Seed;

mod error;
mod seed;

pub mod deriva;
pub mod firma;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

/// Read standard input as string, trimming new lines
pub fn read_stdin() -> Result<String, Error> {
    let mut stdin = std::io::stdin().lock();
    let mut buffer = vec![];
    stdin
        .read_to_end(&mut buffer)
        .map_err(|_| Error::Other("error reading stdin"))?;
    let s = std::str::from_utf8(&buffer).map_err(|_| Error::Other("error reading stdin"))?;

    Ok(s.chars().filter(|c| *c != '\n').collect())
}

/// Read standard input as string, trimming new lines
pub fn read_stdin_seed() -> Result<Seed, Error> {
    let s = read_stdin()?;
    s.parse().map_err(Error::Seed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
