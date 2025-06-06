use virtfw_libefi::efivar::sigdb::EfiSigDB;

const DATA_TEST_DB: &[u8] = include_bytes!("data/test.db.esl");
const DATA_TEST_DBX: &[u8] = include_bytes!("data/test.dbx.esl");

#[test]
fn encode_decode_db() {
    let db = EfiSigDB::new_from_bytes(DATA_TEST_DB).unwrap();
    let bytes: Vec<u8> = (&db).into();
    assert_eq!(DATA_TEST_DB, bytes)
}

#[test]
fn encode_decode_dbx() {
    let dbx = EfiSigDB::new_from_bytes(DATA_TEST_DBX).unwrap();
    let bytes: Vec<u8> = (&dbx).into();
    assert_eq!(DATA_TEST_DBX, bytes)
}

#[test]
fn pk_external_mgmt() {
    let _pk = EfiSigDB::new_pk_external_mgmt();
}
