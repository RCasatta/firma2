use bitcoin::{Address, Amount, Network, Psbt};
use bitcoind::{
    bitcoincore_rpc::{
        json::{AddressType, ImportDescriptors, Timestamp},
        Auth, Client, RpcApi,
    },
    BitcoinD,
};
use firma2_lib::{derive, firma, Seed};
use miniscript::{Descriptor, DescriptorPublicKey};
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
    assert_eq!(DESCRIPTOR_TESTNET, desc.singlesig.bip86_tr.multipath);

    // check every non-multipath descriptor is parsed
    for d in [
        &desc.singlesig.bip44_pkh,
        &desc.singlesig.bip49_shwpkh,
        &desc.singlesig.bip84_wpkh,
        &desc.singlesig.bip86_tr,
    ] {
        // multipath not supported in core
        for e in [&d.external, &d.internal] {
            node.client.get_descriptor_info(e).expect("test");
        }
    }

    let desc_parsed: Descriptor<DescriptorPublicKey> =
        desc.singlesig.bip86_tr.multipath.parse().expect("test");

    let external = desc.singlesig.bip86_tr.external;
    let internal = desc.singlesig.bip86_tr.internal;

    assert_eq!(DESCRIPTOR_TESTNET_EXTERNAL, &external);
    assert_eq!(DESCRIPTOR_TESTNET_INTERNAL, &internal);

    let desc_client = create_blank_wallet(&node, "desc");

    import_descriptor(&desc_client, &external, false);
    import_descriptor(&desc_client, &internal, true);

    let first = get_new_bech32m_address(&desc_client);
    assert_eq!(FIRST_ADDRESS_REGTEST, first.to_string());

    node.client.generate_to_address(1, &first).expect("test");
    node.client
        .generate_to_address(100, &node_address)
        .expect("test");

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

    let params = firma::Params {
        descriptor: desc_parsed,
        psbts: vec![f.path().to_path_buf()],
        network: Network::Regtest,
    };
    let tx = firma::main(&seed, params).expect("test").remove(0).tx();

    let result = desc_client.test_mempool_accept(&[&tx]).expect("test");
    assert!(result[0].allowed);

    desc_client.send_raw_transaction(&tx).expect("test");

    let balances = desc_client.get_balances().expect("test");

    assert_eq!(
        balances.mine.trusted,
        Amount::ONE_BTC * 50 - fee - sent_back
    );
}

fn get_new_bech32m_address(client: &Client) -> Address {
    let address = client
        .get_new_address(None, Some(AddressType::Bech32m))
        .expect("test");
    address.require_network(Network::Regtest).expect("test")
}

fn import_descriptor(client: &Client, descriptor: &str, internal: bool) {
    client
        .import_descriptors(ImportDescriptors {
            descriptor: descriptor.to_owned(),
            timestamp: Timestamp::Now,
            active: Some(true),
            range: None,
            next_index: None,
            internal: Some(internal),
            label: None,
        })
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
