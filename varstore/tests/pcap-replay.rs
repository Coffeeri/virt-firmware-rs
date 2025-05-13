#![cfg(feature = "std")]

use virtfw_libefi::efivar::types::EfiVar;
use virtfw_varstore::json::JsonEfiVarStore;
use virtfw_varstore::pcap::parse::VarstorePcapParser;
use virtfw_varstore::pcap::replay::replay_pcap;
use virtfw_varstore::store::EfiVarStore;

const DATA_FEDORA_BOOT_1_PCAP: &[u8] = include_bytes!("data/fedora-boot-1.pcap.gz");
const DATA_FEDORA_BOOT_1_JSON: &str = include_str!("data/fedora-boot-1.json");
const DATA_FEDORA_BOOT_2_PCAP: &[u8] = include_bytes!("data/fedora-boot-2.pcap.gz");
const DATA_FEDORA_BOOT_2_JSON: &str = include_str!("data/fedora-boot-2.json");

fn compare_store(jstore: JsonEfiVarStore, store: &mut EfiVarStore) {
    store.reset();
    for jvar in jstore.variables {
        let var1 = store.get(&jvar.name, &jvar.guid).unwrap();
        let var2 = &EfiVar::try_from(&jvar).unwrap();
        assert_eq!(var1, var2);
    }
}

#[test]
fn fedora_boot_1() {
    let parser = VarstorePcapParser::new_data_gz(DATA_FEDORA_BOOT_1_PCAP).unwrap();
    let jstore_after = serde_json::from_str::<JsonEfiVarStore>(DATA_FEDORA_BOOT_1_JSON).unwrap();
    let mut store = EfiVarStore::new();
    replay_pcap(parser, &mut store);
    compare_store(jstore_after, &mut store);
}

#[test]
fn fedora_boot_2() {
    let parser = VarstorePcapParser::new_data_gz(DATA_FEDORA_BOOT_2_PCAP).unwrap();
    let jstore_before = serde_json::from_str::<JsonEfiVarStore>(DATA_FEDORA_BOOT_1_JSON).unwrap();
    let jstore_after = serde_json::from_str::<JsonEfiVarStore>(DATA_FEDORA_BOOT_2_JSON).unwrap();
    let mut store = EfiVarStore::new();
    store.json_import(&jstore_before);
    replay_pcap(parser, &mut store);
    compare_store(jstore_after, &mut store);
}
