use bitcoin::Network;
use clap::Parser;
use miniscript::{Descriptor, DescriptorPublicKey};

use crate::{error::Error, seed::Seed};

/// Takes a seed (bip39 or bip93) from standard input, a descriptor and a PSBT. Returns the PSBT signed with details.
#[derive(Parser, Debug)]
#[command(author, version)]
pub struct Params {
    /// Bitcoin Descriptor
    #[clap(short, long, env)]
    pub(crate) descriptor: Descriptor<DescriptorPublicKey>,

    /// Partially Signed Bitcoin Transaction
    pub(crate) psbt: bitcoin::Psbt,

    /// Bitcoin Network
    #[clap(short, long, env)]
    #[arg(default_value_t = Network::Bitcoin)]
    pub(crate) network: Network,
}

pub fn main(seed: Seed, params: Params) -> Result<impl std::fmt::Display, Error> {
    let fingerprint = seed.fingerprint();
    let txid = params
        .psbt
        .extract_tx()
        .map_err(Error::ExtractTx)?
        .compute_txid();
    let network = params.network;

    Ok(format!(
        "fingerprint:{fingerprint:?} txid:{txid} network:{network}"
    ))
}
