use std::str::FromStr;

use bip39::Mnemonic;
use bitcoin::{
    bip32::{Fingerprint, Xpriv},
    secp256k1::{All, Secp256k1},
    Network,
};
use codex32::Codex32String;

#[derive(Debug)]
pub enum SeedError {
    Bip39(bip39::Error),

    Codex32(codex32::Error),

    NeitherMnemonicNorCodex32(String),
}

pub enum Seed {
    Mnemonic(Mnemonic),
    Codex32(Codex32String),
}

impl std::fmt::Display for Seed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Seed::Mnemonic(e) => write!(f, "{}", e),
            Seed::Codex32(e) => write!(f, "{}", e),
        }
    }
}

impl Seed {
    pub fn mnemonic(&self) -> Mnemonic {
        match self {
            Seed::Mnemonic(e) => e.clone(),
            Seed::Codex32(e) => {
                Mnemonic::from_entropy(&e.parts().data()).expect("guaranteed 32 bytes")
            }
        }
    }
    pub fn xprv(&self, network: Network) -> Xpriv {
        let mnemonic = self.mnemonic();
        Xpriv::new_master(network, &mnemonic.to_seed("")).expect("Xpriv fails")
    }

    pub fn fingerprint(&self, secp: &Secp256k1<All>) -> Fingerprint {
        self.xprv(Network::Bitcoin).fingerprint(secp)
    }
}

impl FromStr for Seed {
    type Err = SeedError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(mnemonic) = s.parse::<Mnemonic>() {
            Ok(Seed::Mnemonic(mnemonic))
        } else if let Ok(codex32) = Codex32String::from_string(s.to_string()) {
            Ok(Seed::Codex32(codex32))
        } else {
            Err(SeedError::NeitherMnemonicNorCodex32(s.to_string()))
        }
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use bip39::Mnemonic;
    use bitcoin::{bip32::Xpriv, secp256k1::Secp256k1, Network};
    use codex32::Codex32String;

    use super::Seed;

    pub const MNEMONIC: &str = "flock audit wash crater album salon goose december envelope scissors lock suit render endorse prevent radio expose defy squirrel into grace broken culture burden";
    pub const CODEX_32: &str =
        "ms10leetst9q78hvegp0h6xfpc49asgsdaj9kpya2jkr9pfehf6awv43ep4sqjf0ucdd53raxd";

    #[test]
    fn test_fingerprint() {
        let secp = Secp256k1::new();
        let mnemonic = "episode girl scorpion hope any pave carry rifle limit coffee review bus";
        let expected_fingerprint = "3456016b";
        let mnemonic = Mnemonic::from_str(mnemonic).expect("test");
        let xprv = Xpriv::new_master(Network::Bitcoin, &mnemonic.to_seed("")).expect("test");
        let fingerprint = xprv.fingerprint(&secp);
        assert_eq!(expected_fingerprint, format!("{fingerprint}"));
        let tprv = Xpriv::new_master(Network::Testnet, &mnemonic.to_seed("")).expect("test");
        let fingerprint = tprv.fingerprint(&secp);
        assert_eq!(
            expected_fingerprint,
            format!("{fingerprint}"),
            "network influences fingerprint"
        );
    }

    #[test]
    fn match_39_93() {
        let b39 = Mnemonic::from_str(MNEMONIC).expect("test");
        let b93 = Codex32String::from_string(CODEX_32.to_string()).expect("test");

        assert_eq!(b93.parts().data(), b39.to_entropy());
    }

    #[test]
    fn match_39_93_all_networks() {
        let secp = Secp256k1::new();

        let b39 = Seed::from_str(MNEMONIC).expect("test");
        let b93 = Seed::from_str(CODEX_32).expect("test");

        for network in [
            Network::Bitcoin,
            Network::Testnet,
            Network::Signet,
            Network::Regtest,
        ] {
            assert_eq!(b39.xprv(network), b93.xprv(network));

            assert_eq!(b39.fingerprint(&secp), b93.fingerprint(&secp));
        }
    }
}
