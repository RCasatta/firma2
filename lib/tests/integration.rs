use bitcoin::{Address, Amount, Network, Psbt, Txid};
use bitcoind::{
    bitcoincore_rpc::{
        json::{AddressType, CreateRawTransactionInput},
        Auth, Client, RpcApi,
    },
    BitcoinD,
};
use firma2_lib::{
    import::{self},
    sign, spendable, Seed,
};
use serde_json::Value;
use std::collections::HashMap;
use std::io::Write;
use tempfile::NamedTempFile;

const CODEX_32: &str = include_str!("../../wallet/CODEX_32");
const FIRST_ADDRESS_REGTEST: &str = include_str!("../../wallet/first_address_regtest");

#[test]
fn integration_test() {
    let exe_path = bitcoind::exe_path().expect("test");
    let node = bitcoind::BitcoinD::new(exe_path).expect("test");

    generate_to_own_address(&node, 101);

    let wallet = create_blank_wallet(&node, "test");

    let seed: Seed = CODEX_32.parse().expect("test");

    let desc = import::core_import_json(&seed, bitcoin::Network::Regtest).expect("test");
    let desc_value = serde_json::to_value(desc).unwrap();
    let result = import_descriptors(&wallet, desc_value);

    let result = serde_json::to_string(&result).unwrap();
    assert_eq!(
        result,
        "[{\"success\":true},{\"success\":true},{\"success\":true},{\"success\":true},{\"success\":true},{\"success\":true},{\"success\":true},{\"success\":true}]"
    );

    test(
        &seed,
        &wallet,
        &node,
        AddressType::Bech32m,
        FIRST_ADDRESS_REGTEST,
        "Tr",
    );
    test(
        &seed,
        &wallet,
        &node,
        AddressType::Bech32,
        "bcrt1qrz2fgxvmk5wak7jaju7wgdjdhuh9s7z3q49wya",
        "Wpkh",
    );

    test(
        &seed,
        &wallet,
        &node,
        AddressType::P2shSegwit,
        "2MsjnG76nr1WDDX4Tc2BiGCR9y5Zy7TWnoq",
        "ShWpkh",
    );

    test(
        &seed,
        &wallet,
        &node,
        AddressType::Legacy,
        "mszMRW4zfaRbBi3suqMM4AL217qxqeDtNA",
        "Pkh",
    );
}

fn test(
    seed: &Seed,
    wallet: &Client,
    node: &BitcoinD,
    kind: AddressType,
    expected_addr: &str,
    expected_kind: &str,
) {
    let output = fund_wallet(seed, wallet, node, kind);

    assert_eq!(output.address, expected_addr);
    assert!(output.spendable);
    assert_eq!(output.kind.unwrap(), expected_kind);

    let node_address = generate_to_own_address(node, 1);
    let unspents = wallet.list_unspent(None, None, None, None, None).unwrap();
    assert_eq!(unspents.len(), 1);
    let inputs = create_raw_inputs(&unspents);

    let balances = wallet.get_balances().expect("test");
    let initial_balance = balances.mine.trusted;

    let mut outputs = HashMap::new();
    let sent_back = Amount::ONE_BTC - Amount::from_sat(2_000);
    outputs.insert(node_address.to_string(), sent_back);

    let psbt_result = wallet
        .wallet_create_funded_psbt(&inputs, &outputs, None, None, Some(true))
        .expect(&format!("fail wallet_create_funded_psbt for {kind:?}"));
    let mut f = NamedTempFile::new().expect("test");
    f.as_file_mut()
        .write_all(psbt_result.psbt.as_bytes())
        .expect("Unable to write data");

    let psbt: Psbt = psbt_result.psbt.parse().expect("test");
    let fee = psbt.fee().expect("test");

    let params = sign::Params {
        psbts: vec![f.path().to_path_buf()],
        network: Network::Regtest,
    };
    let signed = sign::main(&seed, params).expect("test").remove(0);
    // dbg!(&signed);
    assert!(
        signed.inputs[0].contains("mine"),
        "not contain mine {kind:?}"
    );
    let tx = signed.tx();

    let result = wallet.test_mempool_accept(&[&tx]).expect("test");
    assert!(result[0].allowed, "not allowed {kind:?}");

    wallet.send_raw_transaction(&tx).expect("test");

    let balances = wallet.get_balances().expect("test");

    println!("done {kind:?}");

    assert_eq!(
        balances.mine.trusted,
        initial_balance - fee - sent_back,
        "{kind:?}",
    );

    let unspents = wallet
        .list_unspent(Some(0), None, None, None, None)
        .unwrap();
    assert_eq!(unspents.len(), 0);

    // test spending 2 inputs
    let _ = fund_wallet(seed, wallet, node, kind);
    let _ = fund_wallet(seed, wallet, node, kind);

    let _ = generate_to_own_address(node, 1);
    let unspents = wallet.list_unspent(None, None, None, None, None).unwrap();
    assert_eq!(unspents.len(), 2);
}

fn generate_to_own_address(node: &BitcoinD, blocks: u64) -> Address {
    let node_address = node
        .client
        .get_new_address(None, None)
        .expect("test")
        .assume_checked();
    node.client
        .generate_to_address(blocks, &node_address)
        .expect("test");
    node_address
}

fn send_to_address(client: &Client, first: &Address) -> Txid {
    client
        .send_to_address(first, Amount::ONE_BTC, None, None, None, None, None, None)
        .unwrap()
}

fn import_descriptors(client: &Client, value: Value) -> Value {
    client
        .call::<Value>("importdescriptors", &[value])
        .expect("test")
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

fn fund_wallet(
    seed: &Seed,
    wallet: &Client,
    node: &BitcoinD,
    kind: AddressType,
) -> spendable::Output {
    let address = wallet
        .get_new_address(Some("ciao"), Some(kind))
        .expect("test");

    let spendable = spendable::main(
        &seed,
        spendable::Params {
            address: address.clone(),
            network: bitcoin::Network::Regtest,
            max: 1000,
        },
    )
    .unwrap();
    let address = address.assume_checked();

    let _txid = send_to_address(&node.client, &address);

    spendable
}

fn create_raw_inputs(
    unspents: &[bitcoind::bitcoincore_rpc::json::ListUnspentResultEntry],
) -> Vec<CreateRawTransactionInput> {
    unspents
        .iter()
        .map(|unspent| CreateRawTransactionInput {
            txid: unspent.txid,
            vout: unspent.vout,
            sequence: None,
        })
        .collect()
}
