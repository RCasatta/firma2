use firma2_lib::{clap::Parser, import, read_stdin_seed};

fn main() {
    let params = import::Params::parse();
    let seed = match read_stdin_seed() {
        Ok(s) => s,
        Err(e) => panic!("{e:?}"),
    };
    match import::main(&seed, params) {
        Ok(s) => println!("{s}"),
        Err(e) => eprintln!("{e:?}"),
    }
}
