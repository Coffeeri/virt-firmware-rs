extern crate std;
use der::{DecodePem, Encode};
use std::boxed::Box;
use std::vec::Vec;
use x509_cert::Certificate;

fn cert_is_pem(data: &[u8]) -> bool {
    let Ok(s) = std::str::from_utf8(data) else {
        return false;
    };
    s.contains("-----BEGIN")
}

pub fn cert_loadfile(filename: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let data = std::fs::read(filename)?;

    if cert_is_pem(&data) {
        let cert = Certificate::from_pem(&data)?;
        Ok(cert.to_der()?)
    } else {
        Ok(data)
    }
}
