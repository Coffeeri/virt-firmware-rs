use log::{error, info};

use virtfw_libefi::efivar::types::EfiVar;
use virtfw_varstore::json::JsonEfiVarStore;

fn loginit() {
    stderrlog::new()
        .module(module_path!())
        .module("virtfw_libefi")
        .module("virtfw_varstore")
        .verbosity(stderrlog::LogLevelNum::Debug)
        .init()
        .unwrap();
}

fn main() {
    let mut args = std::env::args();

    loginit();
    args.next(); // command name
    for filename in args {
        info!("read: {}", filename);
        let Ok(content) = std::fs::read_to_string(filename) else {
            error!("reading file");
            continue;
        };
        let Ok(jstore) = serde_json::from_str::<JsonEfiVarStore>(&content) else {
            error!("parsing file");
            continue;
        };
        for jvar in jstore.variables {
            println!("{}", jvar);
            EfiVar::try_from(jvar).expect("convert failed");
        }
    }
}
