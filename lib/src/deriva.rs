use crate::{error::Error, seed::Seed};
use bitcoin::{bip32::Xpub, key::Secp256k1, Network};
use clap::Parser;

/// Takes a seed from standard input and a path and return the xpub
#[derive(Parser, Debug)]
#[command(author, version)]
pub struct Params {
    /// Derivation path
    pub(crate) path: bitcoin::bip32::DerivationPath,

    /// Bitcoin Network
    #[clap(short, long, env)]
    #[arg(default_value_t = Network::Bitcoin)]
    pub(crate) network: Network,

    #[clap(short, long)]
    pub(crate) p2tr_desc: bool,
}

pub fn main(seed: Seed, params: Params) -> Result<String, Error> {
    let fingerprint = seed.fingerprint().unwrap();
    let Params {
        path,
        network,
        p2tr_desc,
    } = params;
    let secp = Secp256k1::new();
    let xprv = seed
        .xprv(network)
        .unwrap()
        .derive_priv(&secp, &path)
        .unwrap();
    let xpub = Xpub::from_priv(&secp, &xprv);

    let k = format!("[{fingerprint}/{path}]{xpub}");
    if p2tr_desc {
        Ok(format!("tr({k}/<0;1>/*)"))
    } else {
        Ok(k)
    }
}

#[cfg(test)]
mod test {
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
        let seed: Seed = CODEX_32.parse().unwrap();
        let seed_mnemonic: Seed = MNEMONIC.parse().unwrap();
        assert_eq!(seed.fingerprint(), seed_mnemonic.fingerprint());

        let expected =
            format!("[{MASTER_FINGERPRINT}/{BIP86_DERIVATION_PATH_TESTNET}]{MASTER_TPUB}");
        let params = super::Params {
            path: BIP86_DERIVATION_PATH_TESTNET.parse().unwrap(),
            network: bitcoin::Network::Testnet,
            p2tr_desc: false,
        };
        let value = super::main(CODEX_32.parse().unwrap(), params).unwrap();
        assert_eq!(value, expected);

        let expected = format!("[{MASTER_FINGERPRINT}/{BIP86_DERIVATION_PATH}]{MASTER_XPUB}");
        let params = super::Params {
            path: BIP86_DERIVATION_PATH.parse().unwrap(),
            network: bitcoin::Network::Bitcoin,
            p2tr_desc: false,
        };
        let value = super::main(CODEX_32.parse().unwrap(), params).unwrap();
        assert_eq!(value, expected);

        let params = super::Params {
            path: BIP86_DERIVATION_PATH_TESTNET.parse().unwrap(),
            network: bitcoin::Network::Testnet,
            p2tr_desc: true,
        };
        let value = super::main(CODEX_32.parse().unwrap(), params).unwrap();
        assert_eq!(value, DESCRIPTOR_TESTNET);

        let params = super::Params {
            path: BIP86_DERIVATION_PATH.parse().unwrap(),
            network: bitcoin::Network::Bitcoin,
            p2tr_desc: true,
        };
        let value = super::main(CODEX_32.parse().unwrap(), params).unwrap();
        assert_eq!(value, DESCRIPTOR_MAINNET);
    }
}
