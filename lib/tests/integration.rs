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

struct TestContext {
    seed: Seed,
    wallet: Client,
    node: BitcoinD,
    kind: AddressType,

    // node address, it has a different kind than the one we are testing
    node_address: Address,
}

impl TestContext {
    fn new(seed: Seed, wallet: Client, node: BitcoinD, kind: AddressType) -> Self {
        let node_address = generate_to_own_address(&node, 1, other_kind(kind));

        Self {
            seed,
            wallet,
            node,
            kind,
            node_address,
        }
    }

    fn test(&mut self, expected_addr: &str, expected_kind: &str) {
        self.one_input_one_output(expected_addr, expected_kind);
        self.two_inputs_one_output();
        self.one_input_two_outputs();
        self.nlocktime_presigned();
    }

    fn two_inputs_one_output(&self) {
        let _ = fund_wallet(&self.seed, &self.wallet, &self.node, self.kind);
        let _ = fund_wallet(&self.seed, &self.wallet, &self.node, self.kind);

        let balances = self.wallet.get_balances().expect("test");
        assert_eq!(balances.mine.trusted.to_sat(), 200000000, "{:?}", self.kind);

        let mut outputs = HashMap::new();
        outputs.insert(self.node_address.to_string(), Amount::ONE_BTC * 2);
        self.create_and_send_transaction(outputs, 0, subtract_fee_from_first_output(), None);
    }

    fn one_input_one_output(&mut self, expected_addr: &str, expected_kind: &str) {
        let output = fund_wallet(&self.seed, &self.wallet, &self.node, self.kind);

        assert_eq!(output.address, expected_addr);
        assert!(output.spendable);
        assert_eq!(output.kind.unwrap(), expected_kind);

        let mut outputs = HashMap::new();
        outputs.insert(self.node_address.to_string(), Amount::ONE_BTC);

        let _tx =
            self.create_and_send_transaction(outputs, 0, subtract_fee_from_first_output(), None);

        let balances = self.wallet.get_balances().expect("test");
        assert_eq!(balances.mine.trusted.to_sat(), 0, "{:?}", self.kind);
    }

    fn one_input_two_outputs(&mut self) {
        let _ = fund_wallet(&self.seed, &self.wallet, &self.node, self.kind);

        // First transaction with change
        let mut outputs = HashMap::new();
        outputs.insert(self.node_address.to_string(), Amount::ONE_BTC / 2);
        let tx = self.create_and_send_transaction(outputs, 1, None, None);
        assert!(is_same_kind(
            &tx.output[0].script_pubkey,
            &tx.output[1].script_pubkey
        ));

        generate_to_own_address(&self.node, 1, self.kind);

        // Spend the change
        let unspents = self
            .wallet
            .list_unspent(None, None, None, None, None)
            .unwrap();
        let mut outputs = HashMap::new();
        outputs.insert(self.node_address.to_string(), unspents[0].amount);
        self.create_and_send_transaction(outputs, 0, subtract_fee_from_first_output(), None);
    }

    fn nlocktime_presigned(&self) {
        let _ = fund_wallet(&self.seed, &self.wallet, &self.node, self.kind);
        let block_count = self.wallet.get_block_count().unwrap() as i64;
        let locktime = block_count + 10;

        let unspents = self
            .wallet
            .list_unspent(None, None, None, None, None)
            .unwrap();
        let mut outputs = HashMap::new();
        outputs.insert(self.node_address.to_string(), unspents[0].amount);
        let tx = self.create_and_send_transaction(
            outputs,
            0,
            subtract_fee_from_first_output(),
            Some(locktime),
        );
        generate_to_own_address(&self.node, 10, self.kind);
        let result = self.wallet.test_mempool_accept(&[&tx]).expect("test");
        assert!(
            result[0].allowed,
            "not allowed after locktime {:?}",
            self.kind
        );
    }

    #[track_caller]
    fn create_and_send_transaction(
        &self,
        outputs: HashMap<String, Amount>,
        expected_unspents_after: usize,
        options: Option<WalletCreateFundedPsbtOptions>,
        locktime: Option<i64>,
    ) -> bitcoin::Transaction {
        let unspents = self
            .wallet
            .list_unspent(None, None, None, None, None)
            .unwrap();
        let inputs = create_raw_inputs(&unspents);
        assert!(inputs.len() > 0, "no inputs");

        let psbt_result = self
            .wallet
            .wallet_create_funded_psbt(&inputs, &outputs, locktime, options, Some(true))
            .expect(&format!(
                "fail wallet_create_funded_psbt for {:?}",
                self.kind
            ));

        let output = sign_psbt(&self.seed, &psbt_result.psbt);
        let tx = output.tx();

        // Validate and send
        let result = self.wallet.test_mempool_accept(&[&tx]).expect("test");
        if locktime.is_some() {
            assert!(!result[0].allowed, "allowed {:?}", self.kind);
            return tx;
        }
        assert!(result[0].allowed, "not allowed {:?}", self.kind);
        self.wallet.send_raw_transaction(&tx).expect("test");
        generate_to_own_address(&self.node, 1, self.kind);

        // Verify final state
        let unspents = self
            .wallet
            .list_unspent(None, None, None, None, None)
            .unwrap();
        assert_eq!(unspents.len(), expected_unspents_after);

        tx
    }
}

#[test]
fn test_taproot_addresses() {
    let mut test_context = setup(AddressType::Bech32m);
    test_context.test(FIRST_ADDRESS_REGTEST, "Tr");
}

#[test]
fn test_segwit_native_addresses() {
    let mut test_context = setup(AddressType::Bech32);
    test_context.test("bcrt1qrz2fgxvmk5wak7jaju7wgdjdhuh9s7z3q49wya", "Wpkh");
}

#[test]
fn test_segwit_nested_addresses() {
    let mut test_context = setup(AddressType::P2shSegwit);
    test_context.test("2MsjnG76nr1WDDX4Tc2BiGCR9y5Zy7TWnoq", "ShWpkh");
}

#[test]
fn test_legacy_addresses() {
    let mut test_context = setup(AddressType::Legacy);
    test_context.test("mszMRW4zfaRbBi3suqMM4AL217qxqeDtNA", "Pkh");
}

fn setup(kind: AddressType) -> TestContext {
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
    TestContext::new(seed, wallet, node, kind)
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
    let _ = generate_to_own_address(&node, 1, kind);

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
