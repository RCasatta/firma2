use bitcoin::address::NetworkUnchecked;
use bitcoin::key::Secp256k1;
use bitcoin::Address;
use bitcoin::Network;
use clap::Parser;
use miniscript::descriptor::DescriptorType;
use miniscript::{Descriptor, DescriptorPublicKey};
use serde::{Deserialize, Serialize};

use crate::import::compute_descriptors;
use crate::Error;
use crate::Seed;

/// Given a seed and an address tell if we can spend from it from standard descriptors (bip 44,49,84,86)
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

    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,

    pub address: String,
}

pub fn main(seed: &Seed, params: Params) -> Result<Output, Error> {
    let Params {
        address,
        max,
        network,
    } = params;
    let secp = Secp256k1::new();
    let descriptors = compute_descriptors(seed, network, &secp);
    let address = address.require_network(network)?;

    let mut dd = vec![];
    for d in descriptors {
        for definite_desc in d.into_single_descriptors()? {
            dd.push(definite_desc);
        }
    }

    let desc_type = search(&address, &dd, max, network)?;
    let spendable = desc_type.is_some();

    let kind = desc_type.map(|t| format!("{:?}", t));

    Ok(Output {
        spendable,
        kind,
        address: address.to_string(),
    })
}

fn search(
    address: &Address,
    descriptors: &[Descriptor<DescriptorPublicKey>],
    max: u32,
    network: Network,
) -> Result<Option<DescriptorType>, Error> {
    let per_cycle = 10;
    //TODO make this usable for determining if a change is mine and use it in sign for computing my outputs

    // Use ceiling division to ensure we don't miss any addresses
    let cycles = (max + per_cycle - 1) / per_cycle;

    for i in 0..cycles {
        for j in 0..per_cycle {
            let index = i * per_cycle + j;
            if index >= max {
                break;
            }
            for desc in descriptors.iter() {
                let definite_desc = desc.at_derivation_index(index)?;
                let derived_address = definite_desc.address(network)?;
                let desc_type = definite_desc.desc_type();
                if &derived_address == address {
                    return Ok(Some(desc_type));
                }
            }
        }
    }
    Ok(None)
}

#[cfg(test)]
mod test {
    use bitcoin::{Address, Network};
    use std::str::FromStr;

    use crate::Seed;

    use super::Params;

    const CODEX_32: &str = include_str!("../../wallet/CODEX_32");

    #[test]
    fn test_spendable() {
        let seed: Seed = CODEX_32.parse().expect("test");
        let address =
            Address::from_str("tb1pccadr74cd29xf5y0eax2dwnfvjeqwa65c9h09f7cw6c2h6c7rjysrh8wn0")
                .unwrap();

        let result = super::main(
            &seed,
            Params {
                address,
                max: 1000,
                network: Network::Testnet,
            },
        )
        .unwrap();
        assert!(result.spendable);
    }
}
