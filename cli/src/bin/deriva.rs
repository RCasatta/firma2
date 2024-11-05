use firma2_lib::{clap::Parser, deriva};

fn main() {
    let params = deriva::Params::parse();
    match deriva::main(params) {
        Ok(r) => println!("{r}"),
        Err(e) => eprintln!("{e:?}"),
    }
}
