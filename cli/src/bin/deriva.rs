use firma2_lib::{clap::Parser, deriva, read_stdin_seed, serde_json};

fn main() {
    let params = deriva::Params::parse();
    let seed = match read_stdin_seed() {
        Ok(s) => s,
        Err(e) => panic!("{e:?}"),
    };
    match deriva::main(&seed, params) {
        Ok(o) => println!("{}", serde_json::to_string_pretty(&o).unwrap()),
        Err(e) => eprintln!("{e:?}"),
    }
}
