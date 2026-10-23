use der::asn1::ObjectIdentifier;

// x509
pub const OID_CERT_CN: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.5.4.3");
pub const OID_CERT_C: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.5.4.6");
pub const OID_CERT_L: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.5.4.7");
pub const OID_CERT_ST: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.5.4.8");
pub const OID_CERT_O: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.5.4.10");
pub const OID_CERT_OU: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.5.4.11");

// cms
pub const OID_CMS_DATA: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113549.1.7.1");
pub const OID_CMS_SIGNED_DATA: ObjectIdentifier =
    ObjectIdentifier::new_unwrap("1.2.840.113549.1.7.2");

// authenticode
pub const OID_SPC_INDIRECT_DATA: ObjectIdentifier =
    ObjectIdentifier::new_unwrap("1.3.6.1.4.1.311.2.1.4");
pub const OID_SPC_PE_IMAGE_DATA: ObjectIdentifier =
    ObjectIdentifier::new_unwrap("1.3.6.1.4.1.311.2.1.15");

// hashes
pub const OID_ALGO_SHA256: ObjectIdentifier =
    ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.2.1");
pub const OID_ALGO_SHA384: ObjectIdentifier =
    ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.2.2");
pub const OID_ALGO_SHA512: ObjectIdentifier =
    ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.2.3");

// asymmetic crypto
pub const OID_ALGO_RSA: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113549.1.1.1");

// signatures
pub const OID_ALGO_SHA256_RSA: ObjectIdentifier =
    ObjectIdentifier::new_unwrap("1.2.840.113549.1.1.11");
pub const OID_ALGO_SHA384_RSA: ObjectIdentifier =
    ObjectIdentifier::new_unwrap("1.2.840.113549.1.1.12");
pub const OID_ALGO_SHA512_RSA: ObjectIdentifier =
    ObjectIdentifier::new_unwrap("1.2.840.113549.1.1.13");
pub const OID_ALGO_ML_DSA_44: ObjectIdentifier =
    ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.3.17");
pub const OID_ALGO_ML_DSA_65: ObjectIdentifier =
    ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.3.18");
pub const OID_ALGO_ML_DSA_87: ObjectIdentifier =
    ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.3.19");

pub fn oid_to_known_name(oid: &ObjectIdentifier) -> Option<&'static str> {
    match *oid {
        OID_CERT_CN => Some("CN"),
        OID_CERT_C => Some("C"),
        OID_CERT_L => Some("L"),
        OID_CERT_ST => Some("ST"),
        OID_CERT_O => Some("O"),
        OID_CERT_OU => Some("OU"),

        OID_CMS_DATA => Some("Data [cms/pkcs7]"),
        OID_CMS_SIGNED_DATA => Some("SignedData [cms/pkcs7]"),

        OID_SPC_INDIRECT_DATA => Some("SpcIndirectData [authenticode]"),
        OID_SPC_PE_IMAGE_DATA => Some("SpcPeImageData [authenticode]"),

        OID_ALGO_SHA256 => Some("SHA-256"),
        OID_ALGO_SHA384 => Some("SHA-384"),
        OID_ALGO_SHA512 => Some("SHA-512"),

        OID_ALGO_RSA => Some("RSA"),

        OID_ALGO_SHA256_RSA => Some("SHA-256 with RSA"),
        OID_ALGO_SHA384_RSA => Some("SHA-384 with RSA"),
        OID_ALGO_SHA512_RSA => Some("SHA-512 with RSA"),
        OID_ALGO_ML_DSA_44 => Some("ML-DSA-44"),
        OID_ALGO_ML_DSA_65 => Some("ML-DSA-65"),
        OID_ALGO_ML_DSA_87 => Some("ML-DSA-87"),

        _ => None,
    }
}

pub fn oid_to_name(oid: &ObjectIdentifier) -> &'static str {
    oid_to_known_name(oid).unwrap_or("unknown")
}
