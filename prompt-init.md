# Welcome to firma, an offline PSBT signer.

Available commands: deriva, firma, multiqr, jq, vim, age.
Once time setup by creating an password encrypted file `SEED.age` file with: `cat - | age -e -p > SEED.age` and inputting the `SEED` (end with Enter Ctrl-D).
View standard descriptors, or ask a custom derivation with `cat SEED.age | age -d | deriva`
Initialize the `DESCRIPTOR` env var with default pay to taproot key spend `export DESCRIPTOR=$(cat SEED.age | age -d | deriva | jq -r .default)`
Sign a psbt with `cat SEED.age | age -d | firma psbt-file | tee signed-psbt-file`
Export the psbt `cat signed-psbt-file | jq -r .[0].tx | multiqr`
Export the encrypted seed `cat SEED.age | base32 -w0 | multiqr`
Save `DESCRIPTOR` and `NETWORK` env vars in a `.env` file so that env can be bootrapped with `source .env`