use clap::Parser;

use crate::error::Error;

/// Takes a seed (bip39 or bip93) from standard input, a descriptor and a PSBT. Returns the PSBT signed with details.
#[derive(Parser, Debug)]
#[command(author, version)]
pub struct Params {
    /// Bitcoin Descriptor
    #[clap(short, long, env)]
    descriptor: String,

    /// Partially Signed Bitcoin Transaction
    psbt: String,
}

pub fn main(params: Params) -> Result<String, Error> {
    Ok(format!("{params:?}"))
}
