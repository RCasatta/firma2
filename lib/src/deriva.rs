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
}

pub fn main(seed: Seed, params: Params) -> Result<String, Error> {
    let fingerprint = seed.fingerprint().unwrap();
    let path = &params.path;
    let network = params.network;
    let secp = Secp256k1::new();
    let xprv = seed
        .xprv(network)
        .unwrap()
        .derive_priv(&secp, &path)
        .unwrap();
    let xpub = Xpub::from_priv(&secp, &xprv);

    Ok(format!("[{fingerprint}/{path:#}]{xpub}"))
}

#[cfg(test)]
mod test {

    #[test]
    fn test_deriva() {
        let expected = "[01e0b4da/86'/1'/0']tpubDCDuxkQNjPhqtcXWhKr72fwXdaogxop25Dxc5zbWAfNH8Ca7CNRjTeSYqZVA87gW4e8MY9ZcgNCMYrBLyGSRzrCJfEwh6ekK81A2KQPwn4X";
        let seed = "ms10leetst9q78hvegp0h6xfpc49asgsdaj9kpya2jkr9pfehf6awv43ep4sqjf0ucdd53raxd"
            .parse()
            .unwrap();
        let path = "86h/1h/0h".parse().unwrap();
        let network = bitcoin::Network::Testnet;
        let params = super::Params { path, network };
        let value = super::main(seed, params).unwrap();
        assert_eq!(value, expected);
    }
}
