use firma2_lib::{clap::Parser, firma};

fn main() {
    let params = firma::Params::parse();
    match firma::main(params) {
        Ok(r) => println!("{r}"),
        Err(e) => eprintln!("{e:?}"),
    }
}
