use crate::debug_to_string;
use crate::import::compute_descriptors;
use crate::{error::Error, seed::Seed};

use bitcoin::bip32::{ChildNumber, DerivationPath, Fingerprint};
use bitcoin::hex::FromHex;
use bitcoin::psbt::SigningKeys;

use bitcoin::script::PushBytes;
use bitcoin::secp256k1::All;
use bitcoin::{
    consensus::{encode::serialize_hex, Decodable},
    key::Secp256k1,
    Network, Psbt, Transaction, Txid, Witness,
};
use bitcoin::{script, Address, Script, TapLeafHash};
use clap::Parser;
use miniscript::{Descriptor, DescriptorPublicKey};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;

pub type TapKeyOrigin =
    BTreeMap<bitcoin::XOnlyPublicKey, (Vec<TapLeafHash>, (Fingerprint, DerivationPath))>;

/// Takes a seed (bip39 or bip93) from standard input and 1+ PSBT. Computes the standard descriptors an try to sign PSBTs with details.
#[derive(Parser, Debug)]
#[command(author, version)]
pub struct Params {
    /// Files containing Partially Signed Bitcoin Transactions in base64 or binary format
    #[clap(name = "psbt")]
    pub psbts: Vec<PathBuf>,

    /// Bitcoin Network. bitcoin,testnet,signet are possible values
    #[clap(short, long, env)]
    pub network: Network,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
    /// Transaction in hex
    pub tx: String,

    /// PSBT in base64
    pub psbt: String,

    /// Transaction hash
    pub txid: Txid,

    /// human readable inputs
    pub inputs: Vec<String>,

    /// human readable outputs
    pub outputs: Vec<String>,

    /// Signatures added to the PSBT
    pub signatures_added: usize,

    /// The absolute fee of the tx
    pub fee: String,

    /// The net balance from the perspective of the given wallet descriptor
    pub bal: String,
}

pub fn main(seed: &Seed, params: Params) -> Result<Vec<Output>, Error> {
    let Params { psbts, network } = params;

    let secp = Secp256k1::new();

    let descriptors = compute_descriptors(seed, network, &secp);

    let xpriv = seed.xprv(network);

    let mut results = vec![];
    let mut data = Vec::new();

    for psbt_file in psbts {
        std::fs::File::open(psbt_file)?
            .read_to_end(&mut data)
            .expect("Unable to read data");
        let mut psbt: Psbt = match Psbt::deserialize(&data[..]) {
            Ok(s) => s,
            Err(_) => {
                let s = std::str::from_utf8(&data)?;
                let s_no_control_char: String = s.chars().filter(|c| !c.is_control()).collect();
                s_no_control_char.parse()?
            }
        };

        // sign
        let signatures = psbt.sign(&xpriv, &secp).map_err(debug_to_string)?;

        let mut signatures_added = 0;
        for inp in signatures.values() {
            signatures_added += match inp {
                SigningKeys::Ecdsa(a) => a.len(),
                SigningKeys::Schnorr(a) => a.len(),
            };
        }

        let mut sum_input = 0;
        let mut sum_output = 0;

        let mut sum_my_input = 0;
        let mut sum_my_output = 0;

        let mut inputs = vec![];
        for input in psbt.inputs.iter() {
            match input.witness_utxo.as_ref() {
                Some(txout) => {
                    let prev_address = Address::from_script(&txout.script_pubkey, network)?;
                    let amount = txout.value.to_sat();
                    let is_mine = is_mine_taproot(
                        &secp,
                        &descriptors,
                        &txout.script_pubkey,
                        &input.tap_key_origins,
                    );
                    sum_input += amount;
                    if is_mine {
                        sum_my_input += amount;
                    }
                    let is_mine = if is_mine { " mine" } else { "" };
                    inputs.push(format!("{amount:>10}:{prev_address}{is_mine}"));
                }
                None => {
                    match input.non_witness_utxo.as_ref() {
                        Some(tx) => {
                            // TODO
                            let amount = tx.output[0].value;
                            sum_input += amount.to_sat();
                        }
                        None => {
                            return Err(Error::Other(
                                "neither witness_utxo nor non_witness_utxo are set",
                            ))
                        }
                    }
                    // TODO is_mine
                    inputs.push(format!("legacy input"));
                }
            }
        }
        let mut outputs = vec![];
        for (psbt_output, txout) in psbt.outputs.iter().zip(psbt.unsigned_tx.output.iter()) {
            let address = Address::from_script(&txout.script_pubkey, network)?;
            let amount = txout.value.to_sat();
            let is_mine = is_mine_taproot(
                &secp,
                &descriptors,
                &txout.script_pubkey,
                &psbt_output.tap_key_origins,
            );
            sum_output += amount;
            if is_mine {
                sum_my_output += amount;
            }
            let is_mine = if is_mine { " mine" } else { "" };
            outputs.push(format!("{amount:>10}:{address}{is_mine}"));
        }

        for (input, sign_keys) in psbt.inputs.iter_mut().zip(signatures.values()) {
            if input.witness_utxo.is_some() {
                let script_witness = match sign_keys {
                    SigningKeys::Schnorr(_) => {
                        let tap_key_sig = input
                            .tap_key_sig
                            .as_ref()
                            .expect("schnorr sig without tap_key_sig");
                        Witness::p2tr_key_spend(tap_key_sig)
                    }
                    SigningKeys::Ecdsa(sign_keys) => {
                        let sign_key = sign_keys.iter().next().expect("we have one sig");
                        let (_, sig) = input.partial_sigs.iter().next().expect("we have one sig");
                        Witness::p2wpkh(sig, &sign_key.inner)
                    }
                };
                input.final_script_witness = Some(script_witness); // for tr, segwit and nested segwit

                if let Some(redeem_script) = input.redeem_script.as_ref() {
                    // for nested segwit
                    let script_sig = script::Builder::new()
                        .push_slice(<&PushBytes>::try_from(redeem_script.as_bytes()).unwrap())
                        .into_script();
                    input.final_script_sig = Some(script_sig);
                }
            } else {
                let (pubkey, sig) = input.partial_sigs.iter().next().expect("we have one sig");

                let script_sig = script::Builder::new()
                    .push_slice(&sig.serialize())
                    .push_slice(&pubkey.inner.serialize())
                    .into_script();
                input.final_script_sig = Some(script_sig);
            }

            // Clear all the data fields as per the spec.
            input.partial_sigs = BTreeMap::new();
            input.sighash_type = None;
            input.redeem_script = None;
            input.witness_script = None;
            input.bip32_derivation = BTreeMap::new();
        }

        let psbt_base64 = psbt.to_string();
        let tx = psbt.extract_tx()?;
        let txid = tx.compute_txid();
        let tx_hex = serialize_hex(&tx);
        let bal = sum_my_output as i64 - sum_my_input as i64;

        results.push(Output {
            tx: tx_hex,
            psbt: psbt_base64,
            txid,
            inputs,
            outputs,
            fee: format!("{:>10}", sum_input - sum_output),
            bal: format!("{:>10}", bal),
            signatures_added,
        });
    }
    Ok(results)
}

fn is_mine_taproot(
    secp: &Secp256k1<All>,
    descriptor: &[Descriptor<DescriptorPublicKey>],
    script_pubkey: &Script,
    tap_key_origins: &TapKeyOrigin,
) -> bool {
    if let Some(origin) = &tap_key_origins.values().next() {
        is_mine(secp, descriptor, origin.1 .1.clone(), script_pubkey)
    } else {
        false
    }
}

fn is_mine_inner(
    secp: &Secp256k1<All>,
    descriptor: &[Descriptor<DescriptorPublicKey>],
    path: DerivationPath,
    script_pubkey: &Script,
) -> Option<bool> {
    let last = path.into_iter().last();
    if let Some(ChildNumber::Normal { index }) = last {
        for d in descriptor.iter() {
            for d in d.clone().into_single_descriptors().ok()? {
                let derived_script_pubkey =
                    d.derived_descriptor(secp, *index).ok()?.script_pubkey();
                if &derived_script_pubkey == script_pubkey {
                    return Some(true);
                }
            }
        }
    }
    Some(false)
}

fn is_mine(
    secp: &Secp256k1<All>,
    descriptor: &[Descriptor<DescriptorPublicKey>],
    path: DerivationPath,
    script_pubkey: &Script,
) -> bool {
    is_mine_inner(secp, descriptor, path, script_pubkey).unwrap_or(false)
}

impl Output {
    pub fn tx(&self) -> Transaction {
        let bytes = Vec::<u8>::from_hex(&self.tx).expect("guaranteed by invariant");
        Transaction::consensus_decode(&mut &bytes[..]).expect("guaranteed by invariant")
    }
    pub fn psbt(&self) -> Psbt {
        Psbt::from_str(&self.psbt).expect("guaranteed by invariant")
    }
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    use miniscript::{Descriptor, DescriptorPublicKey};

    use crate::seed::Seed;
    use crate::sign::{self, Params};

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
        let seed: Seed = CODEX_32.parse().expect("test");
        let desc: Descriptor<DescriptorPublicKey> = DESCRIPTOR_MAINNET.parse().expect("test");

        // Get the individual xprivs we control. In a real application these would come from a stored secret.
        let master_xpriv = seed.xprv(Network::Bitcoin);
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
            MASTER_FINGERPRINT.parse::<Fingerprint>().expect("test"),
            "m/86'/0'/0'/0/0".parse::<DerivationPath>().expect("test"),
        );
        let origin_input_2 = get_tap_key_origin(
            pk_input_2,
            MASTER_FINGERPRINT.parse::<Fingerprint>().expect("test"),
            "m/86'/0'/0'/1/0".parse::<DerivationPath>().expect("test"),
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

        let unsigned_tx = psbt.clone().extract_tx().expect("test");
        let serialized_unsigned_tx = consensus::encode::serialize_hex(&unsigned_tx);
        assert_eq!(178, serialized_unsigned_tx.len() / 2);

        let mut f = NamedTempFile::new().expect("test");
        f.as_file_mut()
            .write_all(psbt.to_string().as_bytes())
            .expect("Unable to write data");

        // Step 3: Signer role; that signs the PSBT.
        // Step 4: Finalizer role; that finalizes the PSBT.
        // This steps changed in comparison of the original test and unified in the firma::main call
        let params = Params {
            psbts: vec![f.path().to_path_buf()],
            network: Network::Bitcoin,
        };
        let sign::Output { tx, psbt: _, .. } = sign::main(&seed, params).expect("test").remove(0);

        // BOOM! Transaction signed and ready to broadcast.
        assert_eq!(314, tx.len() / 2);

        // check with:
        // bitcoin-cli decoderawtransaction <RAW_TX> true
        println!("Raw Transaction: {}", tx);
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
        let child_xpriv = master_xpriv
            .derive_priv(secp, &derivation_path)
            .expect("test");
        let external_index = ChildNumber::Normal { index: 0 };
        let idx = ChildNumber::from_normal_idx(index).expect("valid index number");

        child_xpriv
            .derive_priv(secp, &[external_index, idx])
            .expect("test")
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
        let child_xpriv = master_xpriv
            .derive_priv(secp, &derivation_path)
            .expect("test");
        let internal_index = ChildNumber::Normal { index: 1 };
        let idx = ChildNumber::from_normal_idx(index).expect("valid index number");

        child_xpriv
            .derive_priv(secp, &[internal_index, idx])
            .expect("test")
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
        let desc = DESCRIPTOR_MAINNET.parse().expect("test");
        let first_address = derive_address(&desc, 0, Network::Bitcoin);
        assert_eq!(FIRST_ADDRESS_MAINNET, first_address.to_string());

        let desc = DESCRIPTOR_TESTNET.parse().expect("test");
        let first_address = derive_address(&desc, 0, Network::Testnet);
        assert_eq!(FIRST_ADDRESS_TESTNET, first_address.to_string());
    }

    fn derive_address(
        desc: &Descriptor<DescriptorPublicKey>,
        index: u32,
        network: Network,
    ) -> bitcoin::Address {
        let d = desc
            .clone()
            .into_single_descriptors()
            .expect("test")
            .remove(0);
        d.at_derivation_index(index)
            .expect("test")
            .address(network)
            .expect("test")
    }
}
