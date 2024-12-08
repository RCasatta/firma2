use bitcoin::address::NetworkUnchecked;
use bitcoin::Address;
use bitcoin::Network;
use clap::Parser;
use miniscript::{Descriptor, DescriptorPublicKey};
use miniscript::descriptor::DescriptorType;
use serde::{Deserialize, Serialize};

use crate::import::compute_from_derive;
use crate::Error;
use crate::Seed;

/// Given a seed and an address tell if we can spend from it from standard descriptors (bip 84,86) (TODO 44,49)
#[derive(Parser, Debug)]
#[command(author, version)]
pub struct Params {
    /// Bitcoin address to check spendability for
    #[clap(short, long)]
    pub address: Address<NetworkUnchecked>,

    /// Generated addresses up to this number
    #[clap(short, long, default_value_t = 1000)]
    pub max: u32,

    /// Bitcoin Network. bitcoin,testnet,signet are possible values
    #[clap(short, long, env)]
    pub network: Network,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
    pub spendable: bool,

    #[serde(skip_serializing_if="Option::is_none")]
    pub kind: Option<String>,
}

pub fn main(seed: &Seed, params: Params) -> Result<Output, Error> {
    let Params {
        address,
        max,
        network,
    } = params;

    let descriptors = compute_from_derive(seed, params.network)?;
    let address = address.require_network(network)?;

    let mut dd = vec![];
    for d in descriptors {
        for definite_desc in d.into_single_descriptors()? {
            dd.push(definite_desc);
        }
    }

    let desc_type = search(address, &dd, max, network)?;
    let spendable = desc_type.is_some();

    let kind = desc_type.map(|t| format!("{:?}", t));
    
    Ok(Output { spendable, kind })
}

fn search(
    address: Address,
    descriptors: &[Descriptor<DescriptorPublicKey>],
    max: u32,
    network: Network,
) -> Result<Option<DescriptorType>, Error> {
    // The address is more likely in the first derivations
    // thus we do a cycle of this number for every descriptor before going to the next

    let per_cycle = 10;
    let cycles = max / per_cycle;

    for i in 0..cycles {
        for j in 0..per_cycle {
            let index = i * per_cycle + j;
            for desc in descriptors.iter() {
                let definite_desc = desc.at_derivation_index(index)?;
                let derived_address = definite_desc.address(network)?;
                let desc_type = definite_desc.desc_type();
                if derived_address == address {
                    return Ok(Some(desc_type));
                }
            }
        }
    }
    Ok(None)
}
