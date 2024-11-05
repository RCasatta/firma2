
# Firma2

PSBT Signer for pay to taproot key spend

## Build

```
nix build

alias firma="$(pwd)/result/bin/firma" #  make shell instead
alias deriva="$(pwd)/result/bin/deriva" #  make shell instead

```

## Example

Enter the wallet directory

```
cd wallet
```

Sign a PSBT

```
export DESCRIPTOR=$(cat descriptor)
cat MNEMONIC.age | age -d | firma $(cat psbt)  # require inputting AGE_PASSPHRASE
```

Derive

```
cat MNEMONIC.age | age -d | deriva m/86h/0h/0h
```
