use clap::Parser;

use crate::{error::Error, read_stdin_seed};

/// Takes a seed from standard input and a path and return the xpub
#[derive(Parser, Debug)]
#[command(author, version)]
pub struct Params {
    /// Derivation path
    path: bitcoin::bip32::DerivationPath,
}

pub fn main(params: Params) -> Result<String, Error> {
    let seed = read_stdin_seed()?;
    let fingerprint = seed.fingerprint();
    Ok(format!("fingerprint:{fingerprint:?} params:{params:?}"))
}
