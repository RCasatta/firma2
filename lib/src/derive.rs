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

/// Takes a seed from standard input and return standard descriptors, or provide custom path for non-standard ones.
#[derive(Parser, Debug)]
#[command(author, version)]
pub struct Params {
    /// Custom derivation path. If not provided standard paths are used (bip49, bip86)
    pub path: Option<bitcoin::bip32::DerivationPath>,

    /// Bitcoin Network. bitcoin,testnet,signet are possible values
    #[clap(short, long, env)]
    pub network: Network,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
    /// Standard singlesig derivations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub singlesig: Option<Singlesig>,

    /// Custom derivation given
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Singlesig {
    /// legacy,
    // pub bip44_pkh: Descriptors, // sign doesn't support legacy

    /// nested segwit
    // pub bip49_shwpkh: Descriptors, // sign doesn't support nested segwit

    /// segwit
    pub bip84_wpkh: Descriptors,

    /// p2tr key spend
    pub bip86_tr: Descriptors,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Descriptors {
    /// A multipath descriptor with <0;1>
    pub multipath: String,

    /// A json that can be used with bitcoin core `importdescriptors` command
    pub import_descriptors: serde_json::Value,
}

pub fn main(seed: &Seed, params: Params) -> Result<Output, Error> {
    let Params { path, network } = params;
    let secp = Secp256k1::new();
    let custom = if let Some(path) = path {
        let xpub_with_origin = xpub_with_origin(seed, network, &secp, path);
        Some(xpub_with_origin)
    } else {
        None
    };
    let singlesig = if custom.is_some() {
        None
    } else {
        Some(Singlesig {
            // bip44_pkh: multi_desc(seed, network, &secp, 44, "pkh"),
            // bip49_shwpkh: multi_desc(seed, network, &secp, 49, "sh(wpkh"),
            bip84_wpkh: multi_desc(seed, network, &secp, 84, "wpkh"),
            bip86_tr: multi_desc(seed, network, &secp, 86, "tr"),
        })
    };

    Ok(Output { singlesig, custom })
}

fn multi_desc(
    seed: &Seed,
    network: Network,
    secp: &Secp256k1<All>,
    bip: u8,
    kind: &str,
) -> Descriptors {
    let external = single_desc(seed, network, &secp, bip, kind, "0").to_string();
    let internal = single_desc(seed, network, &secp, bip, kind, "1").to_string();
    let import_descriptors = import_descriptors(&external, &internal);
    Descriptors {
        multipath: single_desc(seed, network, &secp, bip, kind, "<0;1>").to_string(),
        import_descriptors,
    }
}

fn import_descriptors(external: &str, internal: &str) -> serde_json::Value {
    serde_json::json!(
        [
            {
                "desc": external,
                "timestamp": "now",
                "active": true,
                "internal": false
            },
            {
                "desc": internal,
                "timestamp": "now",
                "active": true,
                "internal": true
            }
        ]
    )
}

fn single_desc(
    seed: &Seed,
    network: Network,
    secp: &Secp256k1<All>,
    bip: u8,
    kind: &str,
    multipath: &str,
) -> Descriptor<DescriptorPublicKey> {
    let network_path = match network {
        Network::Bitcoin => 0,
        _ => 1,
    };
    let path = format!("{bip}'/{network_path}'/0'");
    let path: DerivationPath = path.parse().expect("static path");
    let xpub_with_origin = xpub_with_origin(seed, network, &secp, path);
    let final_parenthesis = if kind.contains('(') { ")" } else { "" };
    let desc_str = format!("{kind}({xpub_with_origin}/{multipath}/*){final_parenthesis}");
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
    const DESCRIPTOR_MAINNET: &str = include_str!("../../wallet/descriptor_mainnet");
    const DESCRIPTOR_TESTNET: &str = include_str!("../../wallet/descriptor_testnet");

    #[test]
    fn test_deriva() {
        let secp = Secp256k1::new();
        let seed: Seed = CODEX_32.parse().expect("test");
        let seed_mnemonic: Seed = MNEMONIC.parse().expect("test");
        assert_eq!(seed.fingerprint(&secp), seed_mnemonic.fingerprint(&secp));

        let expected =
            format!("[{MASTER_FINGERPRINT}/{BIP86_DERIVATION_PATH_TESTNET}]{MASTER_TPUB}");
        let params = super::Params {
            path: Some(BIP86_DERIVATION_PATH_TESTNET.parse().expect("test")),
            network: bitcoin::Network::Testnet,
        };
        let value = super::main(&seed, params).expect("test");
        assert_eq!(value.custom.expect("test"), expected);

        let expected = format!("[{MASTER_FINGERPRINT}/{BIP86_DERIVATION_PATH}]{MASTER_XPUB}");
        let params = super::Params {
            path: Some(BIP86_DERIVATION_PATH.parse().expect("test")),
            network: bitcoin::Network::Bitcoin,
        };
        let value = super::main(&seed, params).expect("test");
        assert_eq!(value.custom.expect("test"), expected);

        let params = super::Params {
            path: None,
            network: bitcoin::Network::Testnet,
        };
        let value = super::main(&seed, params).expect("test");
        assert_eq!(
            value.singlesig.unwrap().bip86_tr.multipath,
            DESCRIPTOR_TESTNET
        );

        let params = super::Params {
            path: None,
            network: bitcoin::Network::Bitcoin,
        };
        let value = super::main(&seed, params).expect("test");
        assert_eq!(
            value.singlesig.unwrap().bip86_tr.multipath,
            DESCRIPTOR_MAINNET
        );
    }
}
