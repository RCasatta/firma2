use firma2_lib::{clap::Parser, deriva, read_stdin_seed, serde_json};

fn main() {
    let params = deriva::Params::parse();
    let seed = match read_stdin_seed() {
        Ok(s) => s,
        Err(e) => panic!("{e:?}"),
    };
    match deriva::main(&seed, params) {
        Ok(o) => {
            let j = serde_json::to_string_pretty(&o).expect("doesn't contain non-string key");
            println!("{j}",)
        }
        Err(e) => eprintln!("{e:?}"),
    }
}
