use firma2_lib::{clap::Parser, firma, read_stdin_seed};

fn main() {
    let params = firma::Params::parse();
    let seed = match read_stdin_seed() {
        Ok(s) => s,
        Err(e) => panic!("{e:?}"),
    };
    match firma::main(seed, params) {
        Ok(r) => println!("{r}"),
        Err(e) => eprintln!("{e:?}"),
    }
}
