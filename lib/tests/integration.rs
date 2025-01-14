use bitcoin::{Address, Amount, Network, Txid};
use bitcoind::{
    bitcoincore_rpc::{
        json::{AddressType, CreateRawTransactionInput, WalletCreateFundedPsbtOptions},
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

struct TestContext<'a> {
    seed: &'a Seed,
    wallet: &'a Client,
    node: &'a BitcoinD,
    kind: AddressType,
    node_address: Option<Address>,
}

impl<'a> TestContext<'a> {
    fn new(seed: &'a Seed, wallet: &'a Client, node: &'a BitcoinD, kind: AddressType) -> Self {
        Self {
            seed,
            wallet,
            node,
            kind,
            node_address: None,
        }
    }

    fn spend_with_change_same_kind(&mut self) {
        let kind = self.kind;
        let node_address = self.node_address.as_ref().expect("node_address not set");
        // Create the change of the same kind of the recipient
        let _ = fund_wallet(self.seed, self.wallet, self.node, self.kind);
        let _ = generate_to_own_address(self.node, 1, self.kind);
        let unspents = self
            .wallet
            .list_unspent(None, None, None, None, None)
            .unwrap();
        assert_eq!(unspents.len(), 1);
        let inputs = create_raw_inputs(&unspents);
        let mut outputs = HashMap::new();
        let sent_back = Amount::ONE_BTC / 2;
        outputs.insert(node_address.to_string(), sent_back);
        let psbt_result = self
            .wallet
            .wallet_create_funded_psbt(&inputs, &outputs, None, None, Some(true))
            .expect(&format!(
                "fail wallet_create_funded_psbt with changefor {kind:?}"
            ));
        let output = sign_psbt(self.seed, &psbt_result.psbt);
        let tx = output.tx();
        assert!(is_same_kind(
            &tx.output[0].script_pubkey,
            &tx.output[1].script_pubkey
        ));
        let result = self.wallet.test_mempool_accept(&[&tx]).expect("test");
        assert!(result[0].allowed, "not allowed {kind:?}");
        self.wallet.send_raw_transaction(&tx).expect("test");
        generate_to_own_address(self.node, 1, self.kind);
        let unspents = self
            .wallet
            .list_unspent(None, None, None, None, None)
            .unwrap();
        assert_eq!(unspents.len(), 1);
        // spend the change to keep the wallet empty
        let inputs = create_raw_inputs(&unspents);
        let mut outputs = HashMap::new();
        outputs.insert(node_address.to_string(), unspents[0].amount);
        let options = subtract_fee_from_first_output();
        let psbt_result = self
            .wallet
            .wallet_create_funded_psbt(&inputs, &outputs, None, options, Some(true))
            .expect(&format!(
                "fail wallet_create_funded_psbt with changefor {kind:?}"
            ));
        let output = sign_psbt(self.seed, &psbt_result.psbt);
        let tx = output.tx();
        let result = self.wallet.test_mempool_accept(&[&tx]).expect("test");
        assert!(result[0].allowed, "not allowed {kind:?}");
        self.wallet.send_raw_transaction(&tx).expect("test");
    }

    fn spend_two_inputs_one_output(&self) {
        let kind = self.kind;
        let node_address = self.node_address.as_ref().expect("node_address not set");
        let _ = fund_wallet(self.seed, self.wallet, self.node, self.kind);
        let _ = fund_wallet(self.seed, self.wallet, self.node, self.kind);
        let _ = generate_to_own_address(self.node, 1, self.kind);
        let unspents = self
            .wallet
            .list_unspent(None, None, None, None, None)
            .unwrap();
        assert_eq!(unspents.len(), 2);
        let balances = self.wallet.get_balances().expect("test");
        assert_eq!(balances.mine.trusted.to_sat(), 200000000, "{kind:?}",);
        let inputs = create_raw_inputs(&unspents);
        let mut outputs = HashMap::new();
        let sent_back = Amount::ONE_BTC * 2;
        outputs.insert(node_address.to_string(), sent_back);
        let options = subtract_fee_from_first_output();
        let psbt_result = self
            .wallet
            .wallet_create_funded_psbt(&inputs, &outputs, None, options, Some(true))
            .expect(&format!(
                "fail wallet_create_funded_psbt with 2 inputs for {kind:?}"
            ));
        let output = sign_psbt(self.seed, &psbt_result.psbt);
        let tx = output.tx();
        let result = self.wallet.test_mempool_accept(&[&tx]).expect("test");
        assert!(result[0].allowed, "not allowed {kind:?}");
        self.wallet.send_raw_transaction(&tx).expect("test");
        let unspents = self
            .wallet
            .list_unspent(None, None, None, None, None)
            .unwrap();
        assert_eq!(unspents.len(), 0);
    }

    fn spend_one_input_one_output(&mut self, expected_addr: &str, expected_kind: &str) {
        let kind = self.kind;
        let output = fund_wallet(self.seed, self.wallet, self.node, self.kind);
        assert_eq!(output.address, expected_addr);
        assert!(output.spendable);
        assert_eq!(output.kind.unwrap(), expected_kind);
        self.node_address = Some(generate_to_own_address(self.node, 1, other_kind(self.kind)));
        dbg!(&self.node_address);
        let unspents = self
            .wallet
            .list_unspent(None, None, None, None, None)
            .unwrap();
        assert_eq!(unspents.len(), 1);
        let inputs = create_raw_inputs(&unspents);
        let balances = self.wallet.get_balances().expect("test");
        let initial_balance = balances.mine.trusted;
        assert_eq!(initial_balance.to_sat(), 100000000, "{kind:?}");
        let mut outputs = HashMap::new();
        let sent_back = Amount::ONE_BTC;
        outputs.insert(
            self.node_address
                .as_ref()
                .expect("node_address not set")
                .to_string(),
            sent_back,
        );
        let options = subtract_fee_from_first_output();
        let psbt_result = self
            .wallet
            .wallet_create_funded_psbt(&inputs, &outputs, None, options, Some(true))
            .expect(&format!("fail wallet_create_funded_psbt for {kind:?}"));
        let output = sign_psbt(self.seed, &psbt_result.psbt);
        let psbt = output.psbt();
        let fee = psbt.fee().expect("test");
        assert!(fee.to_sat() < 4000, "{kind:?}");
        assert!(
            output.inputs[0].contains("mine"),
            "not contain mine {kind:?}"
        );
        let tx = output.tx();
        let result = self.wallet.test_mempool_accept(&[&tx]).expect("test");
        assert!(result[0].allowed, "not allowed {kind:?}");
        self.wallet.send_raw_transaction(&tx).expect("test");
        let balances = self.wallet.get_balances().expect("test");
        println!("done {kind:?}");
        assert_eq!(balances.mine.trusted.to_sat(), 0, "{kind:?}",);
        let unspents = self
            .wallet
            .list_unspent(Some(0), None, None, None, None)
            .unwrap();
        assert_eq!(unspents.len(), 0);
    }
}

#[test]
fn integration_test() {
    let exe_path = bitcoind::exe_path().expect("test");
    let node = bitcoind::BitcoinD::new(exe_path).expect("test");

    generate_to_own_address(&node, 101, AddressType::Bech32m);

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
    let mut ctx = TestContext::new(seed, wallet, node, kind);

    ctx.spend_one_input_one_output(expected_addr, expected_kind);
    ctx.spend_two_inputs_one_output();
    ctx.spend_with_change_same_kind();
}

fn subtract_fee_from_first_output() -> Option<WalletCreateFundedPsbtOptions> {
    Some(WalletCreateFundedPsbtOptions {
        subtract_fee_from_outputs: vec![0],
        ..Default::default()
    })
}

fn generate_to_own_address(node: &BitcoinD, blocks: u64, kind: AddressType) -> Address {
    let node_address = node
        .client
        .get_new_address(None, Some(kind))
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

fn sign_psbt(seed: &Seed, psbt_result: &str) -> sign::Output {
    let mut f = NamedTempFile::new().expect("test");
    f.as_file_mut()
        .write_all(psbt_result.as_bytes())
        .expect("Unable to write data");

    let params = sign::Params {
        psbts: vec![f.path().to_path_buf()],
        network: Network::Regtest,
    };
    let signed = sign::main(seed, params).expect("test").remove(0);

    signed
}

fn other_kind(kind: AddressType) -> AddressType {
    match kind {
        AddressType::Bech32m => AddressType::Bech32,
        AddressType::Bech32 => AddressType::P2shSegwit,
        AddressType::P2shSegwit => AddressType::Legacy,
        AddressType::Legacy => AddressType::Bech32m,
    }
}

enum ScriptType {
    P2PKH,
    P2SH,
    P2WPKH,
    P2TR,
    Unknown,
}

fn get_script_type(script: &bitcoin::ScriptBuf) -> ScriptType {
    if script.is_p2pkh() {
        ScriptType::P2PKH
    } else if script.is_p2sh() {
        ScriptType::P2SH
    } else if script.is_p2wpkh() {
        ScriptType::P2WPKH
    } else if script.is_p2tr() {
        ScriptType::P2TR
    } else {
        ScriptType::Unknown
    }
}

fn is_same_kind(script1: &bitcoin::ScriptBuf, script2: &bitcoin::ScriptBuf) -> bool {
    std::mem::discriminant(&get_script_type(script1))
        == std::mem::discriminant(&get_script_type(script2))
}
