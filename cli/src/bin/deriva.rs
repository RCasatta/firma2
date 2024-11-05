use firma2_lib::{clap::Parser, deriva, read_stdin_seed};

fn main() {
    let params = deriva::Params::parse();
    let seed = match read_stdin_seed() {
        Ok(s) => s,
        Err(e) => panic!("{e:?}"),
    };
    match deriva::main(&seed, params) {
        Ok(r) => println!("{r}"),
        Err(e) => eprintln!("{e:?}"),
    }
}
