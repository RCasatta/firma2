use bitcoin::{key::Secp256k1, Network};
use clap::Parser;
use miniscript::{Descriptor, DescriptorPublicKey};

use crate::{error::Error, seed::Seed};

/// Takes a seed (bip39 or bip93) from standard input, a p2tr key spend descriptor and a PSBT. Returns the PSBT signed with details.
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

pub fn main(seed: Seed, params: Params) -> Result<String, Error> {
    let Params {
        descriptor: _, // necessary for psbt details
        mut psbt,
        network,
    } = params;
    let xpriv = seed.xprv(network).unwrap();
    let secp = Secp256k1::new();
    psbt.sign(&xpriv, &secp).unwrap();

    Ok(psbt.to_string())
}

#[cfg(test)]
mod test {

    // const BIP86_DERIVATION_PATH: &str = include_str!("../../wallet/bip86_derivation_path");
    // const MASTER_FINGERPRINT: &str = include_str!("../../wallet/master_fingerprint");

    // based on https://github.com/rust-bitcoin/rust-bitcoin/blob/master/bitcoin/examples/taproot-psbt-simple.rs
    #[test]
    fn test_firma() {}
}
