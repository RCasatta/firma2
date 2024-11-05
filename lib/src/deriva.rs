use clap::Parser;

use crate::error::Error;

/// Takes a seed and a path and return the xpub
#[derive(Parser, Debug)]
#[command(author, version)]
pub struct Params {
    /// Derivation path
    path: bitcoin::bip32::DerivationPath,
}

pub fn main(params: Params) -> Result<String, Error> {
    Ok(format!("{params:?}"))
}
