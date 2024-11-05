#[derive(Debug)]
pub enum Error {
    Other(&'static str),

    Mnemonic(bip39::Error),

    Seed(crate::seed::SeedError),
}
