use bitcoin::{key::Secp256k1, Network, Psbt};
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

pub fn main(seed: Seed, params: Params) -> Result<Psbt, Error> {
    let Params {
        descriptor: _, // necessary for psbt details
        mut psbt,
        network,
    } = params;
    let xpriv = seed.xprv(network).unwrap();
    let secp = Secp256k1::new();
    psbt.sign(&xpriv, &secp).unwrap();

    Ok(psbt)
}

#[cfg(test)]
mod test {

    const BIP86_DERIVATION_PATH: &str = include_str!("../../wallet/bip86_derivation_path");
    // const BIP86_DERIVATION_PATH_TESTNET: &str =
    //     include_str!("../../wallet/bip86_derivation_path_testnet");
    const MASTER_FINGERPRINT: &str = include_str!("../../wallet/master_fingerprint");
    // const MASTER_XPUB: &str = include_str!("../../wallet/master_xpub");
    // const MASTER_TPUB: &str = include_str!("../../wallet/master_tpub");
    const CODEX_32: &str = include_str!("../../wallet/CODEX_32");
    // const MNEMONIC: &str = include_str!("../../wallet/MNEMONIC");
    const DESCRIPTOR_MAINNET: &str = include_str!("../../wallet/descriptor_mainnet");
    const DESCRIPTOR_TESTNET: &str = include_str!("../../wallet/descriptor_testnet");
    const FIRST_ADDRESS_MAINNET: &str = include_str!("../../wallet/first_address_mainnet");
    const FIRST_ADDRESS_TESTNET: &str = include_str!("../../wallet/first_address_testnet");

    use bitcoin::bip32::{ChildNumber, Xpriv};
    use bitcoin::key::UntweakedPublicKey;
    use bitcoin::psbt::Input;
    use bitcoin::secp256k1::Signing;
    use bitcoin::{
        absolute,
        bip32::{DerivationPath, Fingerprint, Xpub},
        hashes::Hash,
        key::Secp256k1,
        transaction, Amount, Network, OutPoint, Psbt, ScriptBuf, Sequence, TapSighashType,
        Transaction, TxIn, TxOut, Txid, Witness,
    };
    use bitcoin::{consensus, Address, TapLeafHash, XOnlyPublicKey};
    use std::collections::BTreeMap;

    use miniscript::{Descriptor, DescriptorPublicKey};

    use crate::firma::Params;
    use crate::seed::Seed;

    // The dummy UTXO amounts we are spending.
    const DUMMY_UTXO_AMOUNT_INPUT_1: Amount = Amount::from_sat(20_000_000);
    const DUMMY_UTXO_AMOUNT_INPUT_2: Amount = Amount::from_sat(10_000_000);

    // The amounts we are sending to someone, and receiving back as change.
    const SPEND_AMOUNT: Amount = Amount::from_sat(25_000_000);
    const CHANGE_AMOUNT: Amount = Amount::from_sat(4_990_000); // 10_000 sat fee.

    // based on https://github.com/rust-bitcoin/rust-bitcoin/blob/master/bitcoin/examples/taproot-psbt-simple.rs // simple, LOL
    #[test]
    fn test_firma() {
        let secp = Secp256k1::new();
        let seed: Seed = CODEX_32.parse().unwrap();
        let desc: Descriptor<DescriptorPublicKey> = DESCRIPTOR_MAINNET.parse().unwrap();

        // Get the individual xprivs we control. In a real application these would come from a stored secret.
        let master_xpriv = seed.xprv(Network::Bitcoin).unwrap();
        let xpriv_input_1 = get_external_address_xpriv(&secp, master_xpriv, 0);
        let xpriv_input_2 = get_internal_address_xpriv(&secp, master_xpriv, 0);
        let xpriv_change = get_internal_address_xpriv(&secp, master_xpriv, 1);

        // Get the PKs
        let (pk_input_1, _) = Xpub::from_priv(&secp, &xpriv_input_1)
            .public_key
            .x_only_public_key();
        let (pk_input_2, _) = Xpub::from_priv(&secp, &xpriv_input_2)
            .public_key
            .x_only_public_key();
        let (pk_change, _) = Xpub::from_priv(&secp, &xpriv_change)
            .public_key
            .x_only_public_key();

        // Get the Tap Key Origins
        // Map of tap root X-only keys to origin info and leaf hashes contained in it.
        let origin_input_1 = get_tap_key_origin(
            pk_input_1,
            MASTER_FINGERPRINT.parse::<Fingerprint>().unwrap(),
            "m/86'/0'/0'/0/0".parse::<DerivationPath>().unwrap(),
        );
        let origin_input_2 = get_tap_key_origin(
            pk_input_2,
            MASTER_FINGERPRINT.parse::<Fingerprint>().unwrap(),
            "m/86'/0'/0'/1/0".parse::<DerivationPath>().unwrap(),
        );
        let origins = [origin_input_1, origin_input_2];

        // Get the unspent outputs that are locked to the key above that we control.
        // In a real application these would come from the chain.
        let dummy_unspent_transaction_outputs = dummy_unspent_transaction_outputs(&desc);
        let utxos: Vec<TxOut> = dummy_unspent_transaction_outputs
            .clone()
            .into_iter()
            .map(|(_, utxo)| utxo)
            .collect();

        // Get the addresses to send to.
        let address = receivers_address();

        // The inputs for the transaction we are constructing.
        let inputs: Vec<TxIn> = dummy_unspent_transaction_outputs
            .into_iter()
            .map(|(outpoint, _)| TxIn {
                previous_output: outpoint,
                script_sig: ScriptBuf::default(),
                sequence: Sequence::ENABLE_LOCKTIME_NO_RBF,
                witness: Witness::default(),
            })
            .collect();

        // The spend output is locked to a key controlled by the receiver.
        let spend = TxOut {
            value: SPEND_AMOUNT,
            script_pubkey: address.script_pubkey(),
        };

        // The change output is locked to a key controlled by us.
        let change = TxOut {
            value: CHANGE_AMOUNT,
            script_pubkey: ScriptBuf::new_p2tr(&secp, pk_change, None), // Change comes back to us.
        };

        // The transaction we want to sign and broadcast.
        let unsigned_tx = Transaction {
            version: transaction::Version::TWO,  // Post BIP 68.
            lock_time: absolute::LockTime::ZERO, // Ignore the locktime.
            input: inputs,                       // Input is 0-indexed.
            output: vec![spend, change],         // Outputs, order does not matter.
        };

        // Now we'll start the PSBT workflow.
        // Step 1: Creator role; that creates,
        // and add inputs and outputs to the PSBT.
        let mut psbt = Psbt::from_unsigned_tx(unsigned_tx).expect("could not create PSBT");

        // Step 2:Updater role; that adds additional
        // information to the PSBT.
        let ty = TapSighashType::All.into();
        psbt.inputs = vec![
            Input {
                witness_utxo: Some(utxos[0].clone()),
                tap_key_origins: origins[0].clone(),
                tap_internal_key: Some(pk_input_1),
                sighash_type: Some(ty),
                ..Default::default()
            },
            Input {
                witness_utxo: Some(utxos[1].clone()),
                tap_key_origins: origins[1].clone(),
                tap_internal_key: Some(pk_input_2),
                sighash_type: Some(ty),
                ..Default::default()
            },
        ];

        let unsigned_tx = psbt.clone().extract_tx().unwrap();
        let serialized_unsigned_tx = consensus::encode::serialize_hex(&unsigned_tx);
        assert_eq!(356, serialized_unsigned_tx.len());

        // Step 3: Signer role; that signs the PSBT.
        let params = Params {
            descriptor: desc,
            psbt,
            network: Network::Bitcoin,
        };
        let mut psbt = super::main(seed, params).unwrap();

        // Step 4: Finalizer role; that finalizes the PSBT.
        psbt.inputs.iter_mut().for_each(|input| {
            let script_witness = Witness::p2tr_key_spend(&input.tap_key_sig.unwrap());
            input.final_script_witness = Some(script_witness);

            // Clear all the data fields as per the spec.
            input.partial_sigs = BTreeMap::new();
            input.sighash_type = None;
            input.redeem_script = None;
            input.witness_script = None;
            input.bip32_derivation = BTreeMap::new();
        });

        // BOOM! Transaction signed and ready to broadcast.
        let signed_tx = psbt.extract_tx().expect("valid transaction");
        let serialized_signed_tx = consensus::encode::serialize_hex(&signed_tx);
        assert_eq!(628, serialized_signed_tx.len());

        println!("Transaction Details: {:#?}", signed_tx);
        // check with:
        // bitcoin-cli decoderawtransaction <RAW_TX> true
        println!("Raw Transaction: {}", serialized_signed_tx);
    }

    // The dummy unspent transaction outputs that we control.
    fn dummy_unspent_transaction_outputs(
        desc: &Descriptor<DescriptorPublicKey>,
    ) -> Vec<(OutPoint, TxOut)> {
        let script_pubkey_1 = derive_address(desc, 0, Network::Bitcoin).script_pubkey();

        // Arbitrary invalid dummy value.
        let out_point_1 = OutPoint {
            txid: Txid::from_byte_array([0xFF; 32]),
            vout: 0,
        };
        let utxo_1 = TxOut {
            value: DUMMY_UTXO_AMOUNT_INPUT_1,
            script_pubkey: script_pubkey_1,
        };
        let script_pubkey_2 = derive_address(desc, 1, Network::Bitcoin).script_pubkey();

        let out_point_2 = OutPoint {
            txid: Txid::from_byte_array([0xFF; 32]),
            vout: 1,
        };
        let utxo_2 = TxOut {
            value: DUMMY_UTXO_AMOUNT_INPUT_2,
            script_pubkey: script_pubkey_2,
        };
        vec![(out_point_1, utxo_1), (out_point_2, utxo_2)]
    }

    // Derive the external address xpriv.
    fn get_external_address_xpriv<C: Signing>(
        secp: &Secp256k1<C>,
        master_xpriv: Xpriv,
        index: u32,
    ) -> Xpriv {
        let derivation_path: DerivationPath = BIP86_DERIVATION_PATH
            .parse()
            .expect("valid derivation path");
        let child_xpriv = master_xpriv.derive_priv(secp, &derivation_path).unwrap();
        let external_index = ChildNumber::Normal { index: 0 };
        let idx = ChildNumber::from_normal_idx(index).expect("valid index number");

        child_xpriv
            .derive_priv(secp, &[external_index, idx])
            .unwrap()
    }

    // Derive the internal address xpriv.
    fn get_internal_address_xpriv<C: Signing>(
        secp: &Secp256k1<C>,
        master_xpriv: Xpriv,
        index: u32,
    ) -> Xpriv {
        let derivation_path: DerivationPath = BIP86_DERIVATION_PATH
            .parse()
            .expect("valid derivation path");
        let child_xpriv = master_xpriv.derive_priv(secp, &derivation_path).unwrap();
        let internal_index = ChildNumber::Normal { index: 1 };
        let idx = ChildNumber::from_normal_idx(index).expect("valid index number");

        child_xpriv
            .derive_priv(secp, &[internal_index, idx])
            .unwrap()
    }

    // Get the Taproot Key Origin.
    fn get_tap_key_origin(
        x_only_key: UntweakedPublicKey,
        master_fingerprint: Fingerprint,
        path: DerivationPath,
    ) -> BTreeMap<XOnlyPublicKey, (Vec<TapLeafHash>, (Fingerprint, DerivationPath))> {
        let mut map = BTreeMap::new();
        map.insert(x_only_key, (vec![], (master_fingerprint, path)));
        map
    }

    // The address to send to.
    fn receivers_address() -> Address {
        "bc1p0dq0tzg2r780hldthn5mrznmpxsxc0jux5f20fwj0z3wqxxk6fpqm7q0va"
            .parse::<Address<_>>()
            .expect("a valid address")
            .require_network(Network::Bitcoin)
            .expect("valid address for mainnet")
    }

    #[test]
    fn test_first_address() {
        let desc = DESCRIPTOR_MAINNET.parse().unwrap();
        let first_address = derive_address(&desc, 0, Network::Bitcoin);
        assert_eq!(FIRST_ADDRESS_MAINNET, first_address.to_string());

        let desc = DESCRIPTOR_TESTNET.parse().unwrap();
        let first_address = derive_address(&desc, 0, Network::Testnet);
        assert_eq!(FIRST_ADDRESS_TESTNET, first_address.to_string());
    }

    fn derive_address(
        desc: &Descriptor<DescriptorPublicKey>,
        index: u32,
        network: Network,
    ) -> bitcoin::Address {
        let d = desc.clone().into_single_descriptors().unwrap().remove(0);
        d.at_derivation_index(index)
            .unwrap()
            .address(network)
            .unwrap()
    }
}
