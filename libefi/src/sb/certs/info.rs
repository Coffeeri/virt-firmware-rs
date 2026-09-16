extern crate alloc;
use alloc::format;
use alloc::string::{String, ToString};
use der::asn1::ObjectIdentifier;
use der::Decode;
use x509_cert::name::Name;
use x509_cert::Certificate;

use super::oids::{oid_to_name, OID_CERT_CN};
use crate::types::EfiTime;

pub struct CertInfo {
    pub subject: String,
    pub subject_cn: Option<String>,
    pub issuer: String,
    pub issuer_cn: Option<String>,
    pub not_before: EfiTime,
    pub not_after: EfiTime,
    pub signature: String,
}

impl CertInfo {
    fn attr(name: &Name, oid: &ObjectIdentifier) -> Option<String> {
        for attr in name.0.iter() {
            if attr.0.iter().any(|x| x.oid == *oid) {
                return Some(attr.to_string());
            }
        }
        None
    }

    pub fn new(cert: &Certificate) -> Self {
        let subject = &cert.tbs_certificate.subject;
        let issuer = &cert.tbs_certificate.issuer;
        let not_before = cert.tbs_certificate.validity.not_before.to_date_time();
        let not_after = cert.tbs_certificate.validity.not_after.to_date_time();
        let signature = oid_to_name(&cert.signature_algorithm.oid);
        Self {
            subject: subject.to_string(),
            subject_cn: Self::attr(subject, &OID_CERT_CN),
            issuer: issuer.to_string(),
            issuer_cn: Self::attr(issuer, &OID_CERT_CN),
            not_before: EfiTime::from(&not_before),
            not_after: EfiTime::from(&not_after),
            signature: format!("{} ({})", signature, cert.signature_algorithm.oid),
        }
    }

    pub fn new_from_der(der: &[u8]) -> Option<Self> {
        let cert = Certificate::from_der(der).ok()?;
        let info = Self::new(&cert);
        Some(info)
    }

    pub fn desc(&self) -> &str {
        if let Some(ref cn) = self.subject_cn {
            cn
        } else {
            &self.subject
        }
    }
}
