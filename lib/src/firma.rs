use bitcoin::Network;
use clap::Parser;
use miniscript::{Descriptor, DescriptorPublicKey};

use crate::{error::Error, read_stdin_seed};

/// Takes a seed (bip39 or bip93) from standard input, a descriptor and a PSBT. Returns the PSBT signed with details.
#[derive(Parser, Debug)]
#[command(author, version)]
pub struct Params {
    /// Bitcoin Descriptor
    #[clap(short, long, env)]
    descriptor: Descriptor<DescriptorPublicKey>,

    /// Partially Signed Bitcoin Transaction
    psbt: bitcoin::Psbt,

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
