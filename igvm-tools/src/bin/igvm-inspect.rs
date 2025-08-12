use log::info;
use std::env;

use igvm::{IgvmDirectiveHeader, IgvmFile};
use virtfw_igvm_tools::inspect::inspect_igvm;
use virtfw_igvm_tools::ovmfmeta::OvmfMeta;

fn resetvector(d: &IgvmDirectiveHeader) -> Option<&[u8]> {
    match d {
        IgvmDirectiveHeader::PageData { gpa, data, .. } => {
            if *gpa == 0xfffff000 {
                Some(data)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn main() {
    stderrlog::new()
        .module(module_path!())
        .verbosity(stderrlog::LogLevelNum::Info)
        .init()
        .unwrap();

    let mut args: Vec<String> = env::args().collect();
    args.remove(0);
    for arg in args {
        info!("{arg}");

        let blob = std::fs::read(arg).expect("file read error");
        let igvm = IgvmFile::new_from_binary(&blob, None).expect("file parse error");

        inspect_igvm(&igvm);

        for page in igvm.directives().iter().filter_map(resetvector) {
            if let Some(ovmfmeta) = OvmfMeta::new(page) {
                ovmfmeta.print();
            }
        }
    }
}
