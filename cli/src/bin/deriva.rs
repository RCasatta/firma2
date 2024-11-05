use clap::Parser;
use firma2_lib::bitcoin;

/// Takes a seed and a path and return the xpub
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[clap(verbatim_doc_comment)]
pub struct Params {
    /// Derivation path
    path: bitcoin::bip32::DerivationPath,
}

fn main() {
    let params = Params::parse();
    println!("{params:?}");
}
