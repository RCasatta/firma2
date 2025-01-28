# Welcome to the CLI PSBT signer.

Available commands: `import`, `sign`, `spendable`, `multiqr`, `jq`, `vim`, `age`, `base32`, `gzip`.
Once time setup by creating an password encrypted file `SEED.age` file with: `cat - | age -e -p > SEED.age` and inputting the `SEED` (end with Enter/Ctrl-D/Ctrl-D, then Enter for the random passphrase).
View standard descriptors, or ask a custom derivation with `cat SEED.age | age -d | derive`
Sign a psbt with `cat SEED.age | age -d | sign psbt-file | tee signed-psbt-file`
Export the signed tx hex `cat signed-psbt-file | jq -r .[0].tx | multiqr`
Export the encrypted seed `cat SEED.age | base32 -w0 | multiqr` to quickly bootstrap a new machine.
Type `shutdown -h now` to stop the machine.