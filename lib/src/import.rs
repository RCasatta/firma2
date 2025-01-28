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
/// bip 44, 49, 84, 86 wallets in bitcoin core as watch-only
#[derive(Parser, Debug)]
#[command(author, version)]
pub struct Params {
    /// Bitcoin Network. bitcoin,testnet,signet are possible values
    #[clap(short, long, env)]
    pub network: Network,

    /// The wallet name to be created in bitcoin core
    #[clap(short, long)]
    pub wallet_name: String,
}

pub fn main(seed: &Seed, params: Params) -> Result<String, Error> {
    let core_net = params.network.to_core_arg();
    let name = params.wallet_name.clone();
    let r = core_import_json(seed, params.network)?;
    let import = serde_json::to_string(&r).expect("doesn't contain non-string key");
    let s1 = format!("bitcoin-cli -chain={core_net} -named createwallet wallet_name=\"{name}\" blank=true disable_private_keys=true");
    let s2 = format!("bitcoin-cli -chain={core_net} importdescriptors '{import}'");
    Ok(format!("{s1}\n{s2}"))
}

pub fn core_import_json(seed: &Seed, network: Network) -> Result<Vec<ImportElement>, Error> {
    let secp = Secp256k1::new();
    let descriptors = compute_descriptors(seed, network, &secp);
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
    let xpub_with_origin = xpub_with_origin(seed, network, secp, path);
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
    let xprv = seed.xprv(network).derive_priv(secp, &path).expect(
        "statistically impossible to hit, Result will be removed in next rust bitcoin version",
    );
    let xpub = Xpub::from_priv(secp, &xprv);
    let xpub_with_origin = format!("[{fingerprint}/{path}]{xpub}");
    xpub_with_origin
}

pub(crate) fn compute_descriptors(
    seed: &Seed,
    network: Network,
    secp: &Secp256k1<All>,
) -> Vec<Descriptor<DescriptorPublicKey>> {
    let bip84 = single_desc(seed, network, secp, 84, "wpkh");
    let bip86 = single_desc(seed, network, secp, 86, "tr");
    let bib49 = single_desc(seed, network, secp, 49, "sh(wpkh");
    let bib44 = single_desc(seed, network, secp, 44, "pkh");

    vec![bib44, bib49, bip84, bip86]
}

#[cfg(test)]
mod test {
    use bitcoin::key::Secp256k1;

    use crate::seed::Seed;

    const CODEX_32: &str = include_str!("../../wallet/CODEX_32");
    const MNEMONIC: &str = include_str!("../../wallet/MNEMONIC");

    #[test]
    fn test_import() {
        let secp = Secp256k1::new();
        let seed: Seed = CODEX_32.parse().expect("test");
        let seed_mnemonic: Seed = MNEMONIC.parse().expect("test");
        assert_eq!(seed.fingerprint(&secp), seed_mnemonic.fingerprint(&secp));

        let name = "prova_mainnet";
        let params = super::Params {
            wallet_name: name.to_string(),
            network: bitcoin::Network::Bitcoin,
        };
        let value = super::main(&seed, params).expect("test");
        assert!(value.contains("xpub"));
        assert!(value.contains(name));
        assert!(!value.contains("tpub"));
        assert!(value.contains("mainnet"));

        let name = "prova_testnet";
        let params = super::Params {
            wallet_name: name.to_string(),
            network: bitcoin::Network::Testnet,
        };
        let value = super::main(&seed, params).expect("test");
        assert!(!value.contains("xpub"));
        assert!(value.contains(name));
        assert!(value.contains("tpub"));
        assert!(value.contains("testnet"));

        let name = "prova_signet";
        let params = super::Params {
            wallet_name: name.to_string(),
            network: bitcoin::Network::Signet,
        };
        let value = super::main(&seed, params).expect("test");
        assert!(!value.contains("xpub"));
        assert!(value.contains(name));
        assert!(value.contains("tpub"));
        assert!(value.contains("signet"));
    }
}

//TODO hardened derivation in the command are an issue, to overcome I replaced those `'` with `'\''`
