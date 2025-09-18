extern crate alloc;
use alloc::string::{String, ToString};
use der::Decode;
use x509_cert::Certificate;

use crate::types::EfiTime;

pub struct CertInfo {
    pub subject: String,
    pub issuer: String,
    pub not_before: EfiTime,
    pub not_after: EfiTime,
}

impl CertInfo {
    pub fn new_from_der(der: &[u8]) -> Option<Self> {
        let cert = Certificate::from_der(&der).ok()?;
        let subject = cert.tbs_certificate.subject;
        let issuer = cert.tbs_certificate.issuer;
        let not_before = cert.tbs_certificate.validity.not_before.to_date_time();
        let not_after = cert.tbs_certificate.validity.not_after.to_date_time();
        let info = Self {
            subject: subject.to_string(),
            issuer: issuer.to_string(),
            not_before: EfiTime::from(&not_before),
            not_after: EfiTime::from(&not_after),
        };
        Some(info)
    }
}
