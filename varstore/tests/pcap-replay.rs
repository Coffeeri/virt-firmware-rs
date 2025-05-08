use virtfw_libefi::efivar::types::EfiVar;
use virtfw_varstore::json::JsonEfiVarStore;
use virtfw_varstore::pcap::parse::VarstorePcapParser;
use virtfw_varstore::pcap::replay::replay_pcap;
use virtfw_varstore::store::EfiVarStore;

const DATA_FEDORA_BOOT_1_PCAP: &[u8] = include_bytes!("data/fedora-boot-1.pcap.gz");
const DATA_FEDORA_BOOT_1_JSON: &str = include_str!("data/fedora-boot-1.json");

fn compare_store(jstore: JsonEfiVarStore, store: &mut EfiVarStore) {
    store.reset();
    for jvar in jstore.variables {
        let var1 = store.get(&jvar.name, &jvar.guid).unwrap();
        let var2 = &EfiVar::try_from(jvar).unwrap();
        assert_eq!(var1, var2);
    }
}

#[test]
fn fedora_boot_1() {
    let mut store = EfiVarStore::new();
    let jstore = serde_json::from_str::<JsonEfiVarStore>(DATA_FEDORA_BOOT_1_JSON).unwrap();
    let parser = VarstorePcapParser::new_data_gz(DATA_FEDORA_BOOT_1_PCAP).unwrap();
    replay_pcap(parser, &mut store);
    compare_store(jstore, &mut store);
}
