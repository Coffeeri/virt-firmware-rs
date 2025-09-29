extern crate alloc;
use alloc::string::{String, ToString};
use der::asn1::ObjectIdentifier;
use der::Decode;
use x509_cert::name::Name;
use x509_cert::Certificate;

use crate::types::EfiTime;

const OID_CN: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.5.4.3");

pub struct CertInfo {
    pub subject: String,
    pub subject_cn: Option<String>,
    pub issuer: String,
    pub issuer_cn: Option<String>,
    pub not_before: EfiTime,
    pub not_after: EfiTime,
}

impl CertInfo {
    fn attr(name: &Name, oid: &ObjectIdentifier) -> Option<String> {
        name.0
            .iter()
            .flat_map(|x| x.0.iter())
            .find(|x| x.oid == *oid)?
            .value
            .decode_as()
            .ok()
    }

    pub fn new_from_der(der: &[u8]) -> Option<Self> {
        let cert = Certificate::from_der(der).ok()?;
        let subject = cert.tbs_certificate.subject;
        let issuer = cert.tbs_certificate.issuer;
        let not_before = cert.tbs_certificate.validity.not_before.to_date_time();
        let not_after = cert.tbs_certificate.validity.not_after.to_date_time();
        let info = Self {
            subject: subject.to_string(),
            subject_cn: Self::attr(&subject, &OID_CN),
            issuer: issuer.to_string(),
            issuer_cn: Self::attr(&issuer, &OID_CN),
            not_before: EfiTime::from(&not_before),
            not_after: EfiTime::from(&not_after),
        };
        Some(info)
    }
}
