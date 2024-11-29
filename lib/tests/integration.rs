use bitcoin::{Address, Amount, Network, Psbt, Txid};
use bitcoind::{
    bitcoincore_rpc::{json::AddressType, Auth, Client, RpcApi},
    BitcoinD,
};
use firma2_lib::{
    addresses,
    derive::{self, Descriptors},
    sign, Seed,
};
use miniscript::{Descriptor, DescriptorPublicKey};
use serde_json::Value;
use std::collections::HashMap;
use std::io::Write;
use tempfile::NamedTempFile;

const CODEX_32: &str = include_str!("../../wallet/CODEX_32");
const BIP86_DERIVATION_PATH_TESTNET: &str =
    include_str!("../../wallet/bip86_derivation_path_testnet");
const DESCRIPTOR_TESTNET: &str = include_str!("../../wallet/descriptor_testnet");
const DESCRIPTOR_TESTNET_EXTERNAL: &str = include_str!("../../wallet/descriptor_testnet_external");
const DESCRIPTOR_TESTNET_INTERNAL: &str = include_str!("../../wallet/descriptor_testnet_internal");
const FIRST_ADDRESS_REGTEST: &str = include_str!("../../wallet/first_address_regtest");

#[test]
fn integration_test() {
    let exe_path = bitcoind::exe_path().expect("test");
    let node = bitcoind::BitcoinD::new(exe_path).expect("test");

    let node_address = node
        .client
        .get_new_address(None, None)
        .expect("test")
        .assume_checked();

    let seed: Seed = CODEX_32.parse().expect("test");

    let params = derive::Params {
        path: Some(BIP86_DERIVATION_PATH_TESTNET.parse().expect("test")),
        network: bitcoin::Network::Regtest,
    };
    let desc = derive::main(&seed, params).expect("test");
    assert!(desc.singlesig.is_none());
    assert!(DESCRIPTOR_TESTNET.contains(&desc.custom.unwrap()));

    let params = derive::Params {
        path: None,
        network: bitcoin::Network::Regtest,
    };
    let desc = derive::main(&seed, params).expect("test");
    assert!(desc.custom.is_none());
    let s = desc.singlesig.unwrap();

    assert_eq!(DESCRIPTOR_TESTNET, &s.bip86_tr.multipath);

    // check every non-multipath descriptor is parsed
    for d in [
        /*&s.bip44_pkh, &s.bip49_shwpkh,*/
        &s.bip84_wpkh,
        &s.bip86_tr,
    ] {
        // multipath not supported in core
        for e in [
            external(&d.import_descriptors),
            internal(&d.import_descriptors),
        ] {
            node.client.get_descriptor_info(&e).expect("test");
        }
    }

    test(
        &s.bip86_tr,
        &node,
        &node_address,
        &seed,
        AddressType::Bech32m,
        &[
            DESCRIPTOR_TESTNET_EXTERNAL,
            DESCRIPTOR_TESTNET_INTERNAL,
            FIRST_ADDRESS_REGTEST,
        ],
    );
    test(
        &s.bip84_wpkh,
        &node,
        &node_address,
        &seed,
        AddressType::Bech32,
        &[
            "wpkh([01e0b4da/84'/1'/0']tpubDDh27ZBN4jMWEm2Bk7WXPTPSQmB6BwcdASzk5PSMRDCtqWRQGStHZ8EGYogXKCCcMQo31kxZ1LFQGbHZNJ5ejciPR5GzPx3qWri4C8yNNKG/0/*)#m44jdhfu",
            "wpkh([01e0b4da/84'/1'/0']tpubDDh27ZBN4jMWEm2Bk7WXPTPSQmB6BwcdASzk5PSMRDCtqWRQGStHZ8EGYogXKCCcMQo31kxZ1LFQGbHZNJ5ejciPR5GzPx3qWri4C8yNNKG/1/*)#2psnszey",
            "bcrt1qrz2fgxvmk5wak7jaju7wgdjdhuh9s7z3q49wya",
        ],
    );
}

fn test(
    descriptors: &Descriptors,
    node: &BitcoinD,
    node_address: &Address,
    seed: &Seed,
    address_type: AddressType,
    expected: &[&str],
) {
    let desc_parsed: Descriptor<DescriptorPublicKey> = descriptors.multipath.parse().expect("test");

    let imp_desc = &descriptors.import_descriptors;
    let external = external(imp_desc);
    let internal = internal(imp_desc);

    let len = descriptors.multipath.len();
    let checksum = &descriptors.multipath[len - 8..];

    assert_eq!(expected[0], external, "{address_type:?}");
    assert_eq!(expected[1], internal, "{address_type:?}");

    let desc_client = create_blank_wallet(&node, checksum);

    import_descriptors(&desc_client, imp_desc.clone());

    let first = get_new_address(&desc_client, address_type);
    assert_eq!(expected[2], first.to_string(), "{address_type:?}");

    let params = addresses::Params {
        descriptor: desc_parsed.clone(),
        start_from: 0,
        number: 1,
        network: Network::Regtest,
    };
    let addr_result = addresses::main(params).unwrap();
    assert_eq!(expected[2], addr_result[0].addresses[0].address);

    node.client.generate_to_address(1, &first).expect("test");

    node.client
        .generate_to_address(100, &node_address)
        .expect("test");

    let balances = desc_client.get_balances().expect("test");
    let initial_balance = balances.mine.trusted;

    let mut outputs = HashMap::new();
    let sent_back = Amount::from_sat(100_000);
    outputs.insert(node_address.to_string(), sent_back);

    let psbt_result = desc_client
        .wallet_create_funded_psbt(&[], &outputs, None, None, Some(true))
        .expect("test");
    let mut f = NamedTempFile::new().expect("test");
    f.as_file_mut()
        .write_all(psbt_result.psbt.as_bytes())
        .expect("Unable to write data");

    let psbt: Psbt = psbt_result.psbt.parse().expect("test");
    let fee = psbt.fee().expect("test");

    let params = sign::Params {
        descriptor: desc_parsed,
        psbts: vec![f.path().to_path_buf()],
        network: Network::Regtest,
    };
    let tx = sign::main(&seed, params).expect("test").remove(0).tx();

    let result = desc_client.test_mempool_accept(&[&tx]).expect("test");
    assert!(result[0].allowed);

    desc_client.send_raw_transaction(&tx).expect("test");

    let balances = desc_client.get_balances().expect("test");

    assert_eq!(
        balances.mine.trusted,
        initial_balance - fee - sent_back,
        "{address_type:?}",
    );
}

fn _send_to_address(client: &Client, first: &Address) -> Txid {
    client
        .send_to_address(
            first,
            Amount::ONE_BTC * 50,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap()
}

fn get_new_address(client: &Client, address_type: AddressType) -> Address {
    let address = client
        .get_new_address(None, Some(address_type))
        .expect("test");
    address.require_network(Network::Regtest).expect("test")
}

fn import_descriptors(client: &Client, value: Value) {
    client
        .call::<Value>("importdescriptors", &[value])
        .expect("test");
}

fn create_blank_wallet(node: &BitcoinD, wallet_name: &str) -> Client {
    let disable_private_keys = true;
    node.client
        .create_wallet(
            wallet_name,
            Some(disable_private_keys),
            Some(true),
            None,
            None,
        )
        .expect("test");
    let url = format!("http://{}/wallet/{}", &node.params.rpc_socket, wallet_name);

    Client::new(&url, Auth::CookieFile(node.params.cookie_file.clone())).expect("test")
}

fn external(import_descriptors: &Value) -> String {
    get(import_descriptors, 0)
}
fn internal(import_descriptors: &Value) -> String {
    get(import_descriptors, 1)
}
fn get(import_descriptors: &Value, i: usize) -> String {
    import_descriptors
        .get(i)
        .unwrap()
        .get("desc")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string()
}
