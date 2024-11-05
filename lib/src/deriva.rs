use crate::{error::Error, read_stdin_seed};
use bitcoin::Network;
use clap::Parser;

/// Takes a seed from standard input and a path and return the xpub
#[derive(Parser, Debug)]
#[command(author, version)]
pub struct Params {
    /// Derivation path
    path: bitcoin::bip32::DerivationPath,

    /// Bitcoin Network
    #[clap(short, long, env)]
    #[arg(default_value_t = Network::Bitcoin)]
    network: Network,
}

pub fn main(params: Params) -> Result<String, Error> {
    let seed = read_stdin_seed()?;
    let fingerprint = seed.fingerprint();
    Ok(format!("fingerprint:{fingerprint:?} params:{params:?}"))
}
