use log::{error, info};

use virtfw_libefi::efivar::types::EfiVar;
use virtfw_varstore::json::JsonEfiVarStore;

fn loginit() {
    let levelfilter = log::LevelFilter::Debug;
    env_logger::Builder::from_default_env()
        .filter_module(module_path!(), levelfilter)
        .filter_module("virtfw_libefi", levelfilter)
        .filter_module("virtfw_varstore", levelfilter)
        .format_timestamp(None)
        .format_target(false)
        .init();
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
            EfiVar::try_from(&jvar).expect("convert failed");
        }
    }
}
