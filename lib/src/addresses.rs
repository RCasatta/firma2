use bitcoin::Network;
use clap::Parser;
use miniscript::{Descriptor, DescriptorPublicKey};
use serde::{Deserialize, Serialize};

use crate::Error;

/// Derive a bunch of addresses from the given descriptor
#[derive(Parser, Debug)]
#[command(author, version)]
pub struct Params {
    /// Bitcoin Descriptor (not multipath)
    #[clap(short, long, env)]
    pub descriptor: Descriptor<DescriptorPublicKey>,

    /// Index to start from for generating addresses
    #[clap(short, long, default_value_t = 0)]
    pub start_from: u32,

    /// Number of adddress generated
    #[clap(short, long, default_value_t = 100)]
    pub number: u32,

    /// Bitcoin Network. bitcoin,testnet,signet are possible values
    #[clap(short, long, env)]
    pub network: Network,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Addresses {
    pub desc: String,
    pub addresses: Vec<AddressIndex>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddressIndex {
    pub address: String,
    pub index: u32,
}

pub fn main(params: Params) -> Result<Vec<Addresses>, Error> {
    let Params {
        descriptor,
        start_from,
        number,
        network,
    } = params;

    let mut result = vec![];

    for definite_desc in descriptor.into_single_descriptors()? {
        let mut addresses = vec![];
        for index in start_from..start_from + number {
            let d = definite_desc.at_derivation_index(index)?;
            let address = d.address(network)?.to_string();
            addresses.push(AddressIndex { address, index });
        }
        result.push(Addresses {
            desc: definite_desc.to_string(),
            addresses,
        });
    }
    Ok(result)
}
