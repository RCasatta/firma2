use crate::{error::Error, seed::Seed};
use bitcoin::{
    bip32::{DerivationPath, Xpub},
    key::Secp256k1,
    secp256k1::All,
    Network,
};
use clap::Parser;
use miniscript::{Descriptor, DescriptorPublicKey};
use serde::{Deserialize, Serialize};

/// Takes a seed from standard input and return a command string to import
/// bip 84, 86, (TODO 44 49) wallets in bitcoin core as watch-only
#[derive(Parser, Debug)]
#[command(author, version)]
pub struct Params {
    /// Bitcoin Network. bitcoin,testnet,signet are possible values
    #[clap(short, long, env)]
    pub network: Network,
}

pub fn main(seed: &Seed, params: Params) -> Result<Vec<ImportElement>, Error> {
    let secp = Secp256k1::new();
    let descriptors = compute_descriptors(seed, params.network, &secp);
    import_descriptors(descriptors)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImportElement {
    pub desc: String,
    pub timestamp: String,
    pub active: bool,
    pub internal: bool,
    pub range: u32,
}

fn import_descriptors(
    descriptors: Vec<Descriptor<DescriptorPublicKey>>,
) -> Result<Vec<ImportElement>, Error> {
    let mut result = vec![];
    for d in descriptors {
        for (i, single) in d.into_single_descriptors()?.iter().enumerate() {
            result.push(ImportElement {
                desc: single.to_string(),
                timestamp: "now".to_string(),
                active: true,
                internal: i > 0,
                range: 1000,
            });
        }
    }
    Ok(result)
}

fn single_desc(
    seed: &Seed,
    network: Network,
    secp: &Secp256k1<All>,
    bip: u8,
    kind: &str,
) -> Descriptor<DescriptorPublicKey> {
    let network_path = match network {
        Network::Bitcoin => 0,
        _ => 1,
    };
    let path = format!("{bip}'/{network_path}'/0'");
    let path: DerivationPath = path.parse().expect("static path");
    let xpub_with_origin = xpub_with_origin(seed, network, &secp, path);
    let final_parenthesis = if kind.contains('(') { ")" } else { "" };
    let desc_str = format!("{kind}({xpub_with_origin}/<0;1>/*){final_parenthesis}");
    let desc: Descriptor<DescriptorPublicKey> = desc_str.parse().expect("static desc");
    desc
}

fn xpub_with_origin(
    seed: &Seed,
    network: Network,
    secp: &Secp256k1<All>,
    path: DerivationPath,
) -> String {
    let fingerprint = seed.fingerprint(secp);
    let xprv = seed.xprv(network).derive_priv(&secp, &path).expect(
        "statistically impossible to hit, Result will be removed in next rust bitcoin version",
    );
    let xpub = Xpub::from_priv(&secp, &xprv);
    let xpub_with_origin = format!("[{fingerprint}/{path}]{xpub}");
    xpub_with_origin
}

pub(crate) fn compute_descriptors(
    seed: &Seed,
    network: Network,
    secp: &Secp256k1<All>,
) -> Vec<Descriptor<DescriptorPublicKey>> {
    let bip84_wpkh = single_desc(seed, network, &secp, 84, "wpkh");
    let bip86_tr = single_desc(seed, network, &secp, 86, "tr");

    vec![bip84_wpkh, bip86_tr]
}

#[cfg(test)]
mod test {
    use bitcoin::key::Secp256k1;

    use crate::seed::Seed;

    const BIP86_DERIVATION_PATH: &str = include_str!("../../wallet/bip86_derivation_path");
    const BIP86_DERIVATION_PATH_TESTNET: &str =
        include_str!("../../wallet/bip86_derivation_path_testnet");
    const MASTER_FINGERPRINT: &str = include_str!("../../wallet/master_fingerprint");
    const MASTER_XPUB: &str = include_str!("../../wallet/master_xpub");
    const MASTER_TPUB: &str = include_str!("../../wallet/master_tpub");
    const CODEX_32: &str = include_str!("../../wallet/CODEX_32");
    const MNEMONIC: &str = include_str!("../../wallet/MNEMONIC");
    // const DESCRIPTOR_MAINNET: &str = include_str!("../../wallet/descriptor_mainnet");
    // const DESCRIPTOR_TESTNET: &str = include_str!("../../wallet/descriptor_testnet");

    #[test]
    fn test_deriva() {
        let secp = Secp256k1::new();
        let seed: Seed = CODEX_32.parse().expect("test");
        let seed_mnemonic: Seed = MNEMONIC.parse().expect("test");
        assert_eq!(seed.fingerprint(&secp), seed_mnemonic.fingerprint(&secp));

        let _expected =
            format!("[{MASTER_FINGERPRINT}/{BIP86_DERIVATION_PATH_TESTNET}]{MASTER_TPUB}");
        let params = super::Params {
            // path: Some(BIP86_DERIVATION_PATH_TESTNET.parse().expect("test")),
            network: bitcoin::Network::Testnet,
        };
        let _value = super::main(&seed, params).expect("test");
        // assert_eq!(value.custom.expect("test"), expected);

        let _expected = format!("[{MASTER_FINGERPRINT}/{BIP86_DERIVATION_PATH}]{MASTER_XPUB}");
        let params = super::Params {
            // path: Some(BIP86_DERIVATION_PATH.parse().expect("test")),
            network: bitcoin::Network::Bitcoin,
        };
        let _value = super::main(&seed, params).expect("test");
        // assert_eq!(value.custom.expect("test"), expected);

        let params = super::Params {
            network: bitcoin::Network::Testnet,
        };
        let _value = super::main(&seed, params).expect("test");
        // assert_eq!(
        //     value.singlesig.unwrap().bip86_tr.multipath,
        //     DESCRIPTOR_TESTNET
        // );

        let params = super::Params {
            network: bitcoin::Network::Bitcoin,
        };
        let _value = super::main(&seed, params).expect("test");
        // assert_eq!(
        //     value.singlesig.unwrap().bip86_tr.multipath,
        //     DESCRIPTOR_MAINNET
        // );
    }
}
