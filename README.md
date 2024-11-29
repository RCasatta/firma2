
# Firma2

PSBT Signer supporting:

* pay to taproot (bip86)
* pay to witness public key hash (bip84)

Can be used on an offline computer, transporting data via QR codes and off-the-shelf barcode readers.

## Example

### Setup with nix

With nixos or [nix](https://nixos.org/download/) tool installed.
Build the project, put executables in path, and set test env vars (NETWORK and DESCRIPTOR)

```sh
nix develop -c $SHELL
cd wallet # contains some test files, like a test MNEMONIC and an unsigned psbt_file
```

### Setup without nix

With [rust](https://www.rust-lang.org/tools/install) installed

```sh
cargo build --release
export PATH=$PATH:$(pwd)/target/release
export NETWORK=testnet
export DESCRIPTOR="tr([01e0b4da/86'/1'/0']tpubDCDuxkQNjPhqtcXWhKr72fwXdaogxop25Dxc5zbWAfNH8Ca7CNRjTeSYqZVA87gW4e8MY9ZcgNCMYrBLyGSRzrCJfEwh6ekK81A2KQPwn4X/<0;1>/*)#mptp6r5k"
cd wallet # contains some test files, like a test MNEMONIC and an unsigned psbt_file
```

**IMPORTANT**

Some commands requires the seed. All the commands requiring the seed are taking it from standard input.

The real word usage is taking the mnemonic encrypted, decrypting and feeding it to the command so that the clear mnemonic is never persisted.

```sh
cat MNENOMIC.age | age -d | command
```

For the sake of demoing commands from now on we use just `cat MNEMONIC` but in production you should use the encryption.


### Derive

Derive standard descriptors (or provide path for a custom one)

```sh
cat MNEMONIC | derive
```

```json
{
  "singlesig": {
    "bip84_wpkh": {
      "multipath": "wpkh([01e0b4da/84'/1'/0']tpubDDh27ZBN4jMWEm2Bk7WXPTPSQmB6BwcdASzk5PSMRDCtqWRQGStHZ8EGYogXKCCcMQo31kxZ1LFQGbHZNJ5ejciPR5GzPx3qWri4C8yNNKG/<0;1>/*)#29tfunwc",
      "external": "wpkh([01e0b4da/84'/1'/0']tpubDDh27ZBN4jMWEm2Bk7WXPTPSQmB6BwcdASzk5PSMRDCtqWRQGStHZ8EGYogXKCCcMQo31kxZ1LFQGbHZNJ5ejciPR5GzPx3qWri4C8yNNKG/0/*)#m44jdhfu",
      "internal": "wpkh([01e0b4da/84'/1'/0']tpubDDh27ZBN4jMWEm2Bk7WXPTPSQmB6BwcdASzk5PSMRDCtqWRQGStHZ8EGYogXKCCcMQo31kxZ1LFQGbHZNJ5ejciPR5GzPx3qWri4C8yNNKG/1/*)#2psnszey"
    },
    "bip86_tr": {
      "multipath": "tr([01e0b4da/86'/1'/0']tpubDCDuxkQNjPhqtcXWhKr72fwXdaogxop25Dxc5zbWAfNH8Ca7CNRjTeSYqZVA87gW4e8MY9ZcgNCMYrBLyGSRzrCJfEwh6ekK81A2KQPwn4X/<0;1>/*)#mptp6r5k",
      "external": "tr([01e0b4da/86'/1'/0']tpubDCDuxkQNjPhqtcXWhKr72fwXdaogxop25Dxc5zbWAfNH8Ca7CNRjTeSYqZVA87gW4e8MY9ZcgNCMYrBLyGSRzrCJfEwh6ekK81A2KQPwn4X/0/*)#awxxyl4x",
      "internal": "tr([01e0b4da/86'/1'/0']tpubDCDuxkQNjPhqtcXWhKr72fwXdaogxop25Dxc5zbWAfNH8Ca7CNRjTeSYqZVA87gW4e8MY9ZcgNCMYrBLyGSRzrCJfEwh6ekK81A2KQPwn4X/1/*)#v6r8e297"
    }
  }
}
```

It's possible to specify a custom path for derivation

```sh
cat MNEMONIC | derive 0h/1h
```

```json
{
  "custom": "[01e0b4da/0'/1']tpubDBteAN9SBvfyvs8raNRRMv3uZf371jGbTUT5CcjR1HzWyByYGnhfRz5PQV6mcg2s1EKtZAnC6EW29NGcQzBBNhKW6VMnmZngcT6kukRGQ6v"
}
```

### Sign a PSBT

```sh
cat MNEMONIC | sign psbt_file # --network testnet --descriptor $DESC if env vars not set
```

```json
[
  {
    "tx": "020000000001015417f4cd7c8f49fe6992ae65413b42cc3d777a999fa51ffd6dfbb96a51c1f5770000000000fdffffff02a0860100000000001600147cc19fbb961bb00f8e5630474e23cf3c4984b82fca65042a01000000225120f6f8dc277f283ba4ec3836874739d377d13cb0b48d8075e84f801936879000e901400fe77fd4130c27d421ac6fd9b310c2dbfb3446668583d00a09a80d53cc55d8440c4045f401498afa8c4eab5e53ef58cbb17f4f3b84afe75173807952131613a500000000",
    "psbt": "cHNidP8BAH0CAAAAAVQX9M18j0n+aZKuZUE7Qsw9d3qZn6Uf/W37uWpRwfV3AAAAAAD9////AqCGAQAAAAAAFgAUfMGfu5YbsA+OVjBHTiPPPEmEuC/KZQQqAQAAACJRIPb43Cd/KDuk7Dg2h0c503fRPLC0jYB16E+AGTaHkADpAAAAAAABASsA8gUqAQAAACJRIMY60fq4aopk0I/PTKa6aWSyB3dUwW7yp9h2sKvrHhyJAQhCAUAP53/UEwwn1CGsb9mzEMLb+zRGZoWD0AoJqA1TzFXYRAxARfQBSYr6jE6rXlPvWMuxf087hK/nUXOAeVITFhOlARNAD+d/1BMMJ9QhrG/ZsxDC2/s0RmaFg9AKCagNU8xV2EQMQEX0AUmK+oxOq15T71jLsX9PO4Sv51FzgHlSExYTpSEWU8hg52nfZN8wuDYlsYEmEsQ9+0AVtsjSvbJMMLgb3RIZAAHgtNpWAACAAQAAgAAAAIAAAAAAAAAAAAEXIFPIYOdp32TfMLg2JbGBJhLEPftAFbbI0r2yTDC4G90SAAABBSBfecobx86k3gNeTd17VEQKE8f/q55Sozbft7xye4eyCiEHX3nKG8fOpN4DXk3de1REChPH/6ueUqM237e8cnuHsgoZAAHgtNpWAACAAQAAgAAAAIABAAAAAAAAAAA=",
    "txid": "a56fb5e42d0ddfa9d817947e1986d8381a4b0746685c27862c34c4dc88f55ca8",
    "inputs": [
      "5000000000:tb1pccadr74cd29xf5y0eax2dwnfvjeqwa65c9h09f7cw6c2h6c7rjysrh8wn0 mine"
    ],
    "outputs": [
      "    100000:tb1q0nqelwukrwcqlrjkxpr5ug7083ycfwp0228y3c",
      "4999898570:tb1p7mudcfml9qa6fmpcx6r5wwwnwlgnev953kq8t6z0sqvndpusqr5st5tmdy mine"
    ],
    "signatures_added": 1,
    "fee": "      1430",
    "bal": "   -101430"
  }
]
```

Note some inputs and outpus are `mine` because the command know the env var `DESCRIPTOR` and can verify ownership.
The `bal` field is the net balance of the transaction from the perspective of the `DESCRIPTOR`.

It's also possible to sign multiple psbts at once

```sh
cat MNEMONIC | sign psbts/psbt*
```

### Addresses

Always with `NETWORK` and `DESCRIPTOR` env var already set

```
addresses --number 2
```

```json
[
  {
    "desc": "tr([01e0b4da/86'/1'/0']tpubDCDuxkQNjPhqtcXWhKr72fwXdaogxop25Dxc5zbWAfNH8Ca7CNRjTeSYqZVA87gW4e8MY9ZcgNCMYrBLyGSRzrCJfEwh6ekK81A2KQPwn4X/0/*)#awxxyl4x",
    "addresses": [
      {
        "address": "tb1pccadr74cd29xf5y0eax2dwnfvjeqwa65c9h09f7cw6c2h6c7rjysrh8wn0",
        "index": 0
      },
      {
        "address": "tb1ps4e34gzelyrt0uvujgz7p5tdjzt7qz8kgnnt4zvle3u8twvhhcfqs7nu9e",
        "index": 1
      }
    ]
  },
  {
    "desc": "tr([01e0b4da/86'/1'/0']tpubDCDuxkQNjPhqtcXWhKr72fwXdaogxop25Dxc5zbWAfNH8Ca7CNRjTeSYqZVA87gW4e8MY9ZcgNCMYrBLyGSRzrCJfEwh6ekK81A2KQPwn4X/1/*)#v6r8e297",
    "addresses": [
      {
        "address": "tb1p7mudcfml9qa6fmpcx6r5wwwnwlgnev953kq8t6z0sqvndpusqr5st5tmdy",
        "index": 0
      },
      {
        "address": "tb1pm9r388z5ljwnm63ssr0t388fxeg9j85u7nn4lgjku9dk6tr20d9qzxxekc",
        "index": 1
      }
    ]
  }
]
```

View only the first external address

```sh
$ addresses | jq -r '.[0].addresses[0].address'
tb1pccadr74cd29xf5y0eax2dwnfvjeqwa65c9h09f7cw6c2h6c7rjysrh8wn0 
```

## Mnemonic

To store the mnemonic encrypted with age use:

```sh
cat - | age -e -p -o MNEMONIC.age
```

`cat -` means to read the data from standard input, by doing so we don't save the clear mnemonic anywhere

`age -e -p -o MNEMONIC.age` means to encrypt `-e` with the `age` tool with a passphrase `-p` and write the result in the file `MNEMONIC.age`

to print the mnemonic:

```sh
cat MNEMONIC.age | age -d # and enter the previously generated password
```

**IMPORTANT**

The age command prints `Enter passphrase (leave empty to autogenerate a secure one):` but the first thing you have to type is the mnemonic, followed by enter, then by `Ctrl-D`, then another enter to generate a passphrase.

## QR codes

Multiple signed transactions can be transported via QR codes, for example with:

```sh
cat result_from_sign | jq '[.[].tx]' | gzip | base32 -w0 | multiqr
```

## Create PSBT

```sh
bitcoin-cli -named walletcreatefundedpsbt inputs='[{"txid":"2e6425eb67549e638503d541fb1e1fb64f01a5d7dd7571a8ed78fac9a689aafe","vout":0}]' outputs='[{"tb1pvsdpz8cucqz4tylmgtemn2qp6l9e8mptn36emnd6w6ntz8p8yp3s69gc7q":0.0001}]'
```

## Misc

Check the shasum of something passing through the pipe without influencing the data

```sh
$ echo ciao | tee >(shasum -a 256 1>&2) | shasum -a 256

6f0378f21a495f5c13247317d158e9d51da45a5bf68fc2f366e450deafdc8302  -
6f0378f21a495f5c13247317d158e9d51da45a5bf68fc2f366e450deafdc8302  -
```