use firma2_lib::{addresses, clap::Parser, serde_json};

fn main() {
    let params = addresses::Params::parse();
    match addresses::main(params) {
        Ok(o) => {
            let j = serde_json::to_string_pretty(&o).expect("doesn't contain non-string key");
            println!("{j}",)
        }
        Err(e) => eprintln!("{e:?}"),
    }
}
