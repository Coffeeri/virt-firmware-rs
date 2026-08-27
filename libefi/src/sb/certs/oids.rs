use der::asn1::ObjectIdentifier;

pub const OID_CERT_CN: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.5.4.3");

// hashes
pub const OID_ALGO_SHA256: ObjectIdentifier =
    ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.2.1");

// signatures
pub const OID_ALGO_SHA256_RSA: ObjectIdentifier =
    ObjectIdentifier::new_unwrap("1.2.840.113549.1.1.11");

pub fn oid_name(oid: &ObjectIdentifier) -> Option<&'static str> {
    match *oid {
        OID_ALGO_SHA256 => Some("SHA-256"),
        OID_ALGO_SHA256_RSA => Some("SHA-256 with RSA"),
        _ => None,
    }
}
