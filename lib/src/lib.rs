use std::io::Read;

pub use bitcoin;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub fn read_stdin() -> Result<String, &'static str> {
    let mut stdin = std::io::stdin().lock();
    let mut buffer = vec![];
    stdin
        .read_to_end(&mut buffer)
        .map_err(|_| "error reading stdin")?;

    Ok(std::str::from_utf8(&buffer)
        .map_err(|_| "error converting stdin to utf8 string")?
        .to_string())
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
