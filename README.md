
# Firma2

PSBT Signer for pay to taproot key spend

## Example

Build and put executables in path

```sh
nix develop
```

Enter the wallet directory

```sh
cd wallet
```

## Derive

Derive standard descriptors (or provide path for a custom one)

```sh
cat MNEMONIC.age | age -d | deriva # input the value in AGE_PASSPHRASE, in a real setup the passphrase should be stored separately

# Or

cat MNEMONIC | deriva # Demo purpose, don't store the mnemonic unencrypted
```

```json
{
  "singlesig": {
    "bip44_pkh": {
      "multipath": "pkh([01e0b4da/44'/0'/0']xpub6C7SKkuuZBozxjHUvKqTUNDPJaPfXbZ9FSz7xBp4JTRepV3rkRiih5u6RXGuErRrerjAaR4DHpYpWaMb2BXSfemxAbkcWgjPfmXE5gm65qP/<0;1>/*)#tp4recgf",
      "external": "pkh([01e0b4da/44'/0'/0']xpub6C7SKkuuZBozxjHUvKqTUNDPJaPfXbZ9FSz7xBp4JTRepV3rkRiih5u6RXGuErRrerjAaR4DHpYpWaMb2BXSfemxAbkcWgjPfmXE5gm65qP/0/*)#mx977k6c",
      "internal": "pkh([01e0b4da/44'/0'/0']xpub6C7SKkuuZBozxjHUvKqTUNDPJaPfXbZ9FSz7xBp4JTRepV3rkRiih5u6RXGuErRrerjAaR4DHpYpWaMb2BXSfemxAbkcWgjPfmXE5gm65qP/1/*)#2jqlrr2q"
    },
    "bip49_shwpkh": {
      "multipath": "sh(wpkh([01e0b4da/49'/0'/0']xpub6D13BByjKNuamAzbUpJFjSvWzmb3id1WieYxeNKJAf4Jsf8Sx3AoAv6V18R7uZbeLKHYRJfmRh7CBQhJhcqRN2Pg3jJu55GvQUc2knXJgwK/<0;1>/*))#tn56d4t8",
      "external": "sh(wpkh([01e0b4da/49'/0'/0']xpub6D13BByjKNuamAzbUpJFjSvWzmb3id1WieYxeNKJAf4Jsf8Sx3AoAv6V18R7uZbeLKHYRJfmRh7CBQhJhcqRN2Pg3jJu55GvQUc2knXJgwK/0/*))#2e54q44q",
      "internal": "sh(wpkh([01e0b4da/49'/0'/0']xpub6D13BByjKNuamAzbUpJFjSvWzmb3id1WieYxeNKJAf4Jsf8Sx3AoAv6V18R7uZbeLKHYRJfmRh7CBQhJhcqRN2Pg3jJu55GvQUc2knXJgwK/1/*))#lc6rc2ql"
    },
    "bip84_wpkh": {
      "multipath": "wpkh([01e0b4da/84'/0'/0']xpub6DLYEYw4nCtz5YM3o7v4zKKeXADrGaWxR7u94RWmXv9XkxnFz9CrKsWPg5pqwB5XwY4itvvUEciTP87D5LWjxxDLtg1APP6QTGQ1BBNyBY4/<0;1>/*)#wvaaw4h2",
      "external": "wpkh([01e0b4da/84'/0'/0']xpub6DLYEYw4nCtz5YM3o7v4zKKeXADrGaWxR7u94RWmXv9XkxnFz9CrKsWPg5pqwB5XwY4itvvUEciTP87D5LWjxxDLtg1APP6QTGQ1BBNyBY4/0/*)#aw4t55ts",
      "internal": "wpkh([01e0b4da/84'/0'/0']xpub6DLYEYw4nCtz5YM3o7v4zKKeXADrGaWxR7u94RWmXv9XkxnFz9CrKsWPg5pqwB5XwY4itvvUEciTP87D5LWjxxDLtg1APP6QTGQ1BBNyBY4/1/*)#v6s2fpmg"
    },
    "bip86_tr": {
      "multipath": "tr([01e0b4da/86'/0'/0']xpub6CPQjH8Lh22qxYN1ZNrCKqx2TwZUcoyb4thNUtLJHDbgasMY4wCv1Njy8FJ1bHEDeQVVUU9NNRMxVGfF5B6xAq5YusQvbDMLAUuAFntWLY4/<0;1>/*)#202k4zmq",
      "external": "tr([01e0b4da/86'/0'/0']xpub6CPQjH8Lh22qxYN1ZNrCKqx2TwZUcoyb4thNUtLJHDbgasMY4wCv1Njy8FJ1bHEDeQVVUU9NNRMxVGfF5B6xAq5YusQvbDMLAUuAFntWLY4/0/*)#64mkrs39",
      "internal": "tr([01e0b4da/86'/0'/0']xpub6CPQjH8Lh22qxYN1ZNrCKqx2TwZUcoyb4thNUtLJHDbgasMY4wCv1Njy8FJ1bHEDeQVVUU9NNRMxVGfF5B6xAq5YusQvbDMLAUuAFntWLY4/1/*)#tp7h79pa"
    }
  },
  "default": "tr([01e0b4da/86'/0'/0']xpub6CPQjH8Lh22qxYN1ZNrCKqx2TwZUcoyb4thNUtLJHDbgasMY4wCv1Njy8FJ1bHEDeQVVUU9NNRMxVGfF5B6xAq5YusQvbDMLAUuAFntWLY4/<0;1>/*)#202k4zmq",
  "custom": null
}
```


Sign a PSBT

```sh
export DESCRIPTOR=$(cat MNEMONIC.age | age -d | deriva | jq -r .default)
cat MNEMONIC.age | age -d | firma psbt  # require inputting AGE_PASSPHRASE
```

```json
[
  {
    "tx": "020000000001015417f4cd7c8f49fe6992ae65413b42cc3d777a999fa51ffd6dfbb96a51c1f5770000000000fdffffff02a0860100000000001600147cc19fbb961bb00f8e5630474e23cf3c4984b82fca65042a01000000225120f6f8dc277f283ba4ec3836874739d377d13cb0b48d8075e84f801936879000e901400fe77fd4130c27d421ac6fd9b310c2dbfb3446668583d00a09a80d53cc55d8440c4045f401498afa8c4eab5e53ef58cbb17f4f3b84afe75173807952131613a500000000",
    "psbt": "cHNidP8BAH0CAAAAAVQX9M18j0n+aZKuZUE7Qsw9d3qZn6Uf/W37uWpRwfV3AAAAAAD9////AqCGAQAAAAAAFgAUfMGfu5YbsA+OVjBHTiPPPEmEuC/KZQQqAQAAACJRIPb43Cd/KDuk7Dg2h0c503fRPLC0jYB16E+AGTaHkADpAAAAAAABASsA8gUqAQAAACJRIMY60fq4aopk0I/PTKa6aWSyB3dUwW7yp9h2sKvrHhyJAQhCAUAP53/UEwwn1CGsb9mzEMLb+zRGZoWD0AoJqA1TzFXYRAxARfQBSYr6jE6rXlPvWMuxf087hK/nUXOAeVITFhOlARNAD+d/1BMMJ9QhrG/ZsxDC2/s0RmaFg9AKCagNU8xV2EQMQEX0AUmK+oxOq15T71jLsX9PO4Sv51FzgHlSExYTpSEWU8hg52nfZN8wuDYlsYEmEsQ9+0AVtsjSvbJMMLgb3RIZAAHgtNpWAACAAQAAgAAAAIAAAAAAAAAAAAEXIFPIYOdp32TfMLg2JbGBJhLEPftAFbbI0r2yTDC4G90SAAABBSBfecobx86k3gNeTd17VEQKE8f/q55Sozbft7xye4eyCiEHX3nKG8fOpN4DXk3de1REChPH/6ueUqM237e8cnuHsgoZAAHgtNpWAACAAQAAgAAAAIABAAAAAAAAAAA=",
    "txid": "a56fb5e42d0ddfa9d817947e1986d8381a4b0746685c27862c34c4dc88f55ca8",
    "inputs": [
      "5000000000:bc1pccadr74cd29xf5y0eax2dwnfvjeqwa65c9h09f7cw6c2h6c7rjys5l3pfq"
    ],
    "outputs": [
      "    100000:bc1q0nqelwukrwcqlrjkxpr5ug7083ycfwp0qvuh2t",
      "4999898570:bc1p7mudcfml9qa6fmpcx6r5wwwnwlgnev953kq8t6z0sqvndpusqr5suua5ht"
    ],
    "signatures_added": 1,
    "fee": "      1430",
    "bal": "         0"
  }
]
```