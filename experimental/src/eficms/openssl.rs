use log::debug;

use openssl::pkcs7::{Pkcs7, Pkcs7Flags};
use openssl::stack::Stack;
use openssl::x509::store::{X509Store, X509StoreBuilder};
use openssl::x509::verify::X509VerifyFlags;
use openssl::x509::{X509PurposeId, X509};

use super::{EfiCms, EfiCmsCert, EfiCmsError};

impl From<openssl::error::ErrorStack> for EfiCmsError {
    fn from(err: openssl::error::ErrorStack) -> Self {
        Self::OpenSSL(err)
    }
}

// ---------------------------------------------------------------------
// OpenSSL variant

#[derive(Default)]
pub struct EfiCmsOpenSSL {
    certs: Vec<(EfiCmsCert, Vec<u8>)>,
}

impl EfiCmsOpenSSL {
    pub fn new() -> Self {
        Self { certs: Vec::new() }
    }

    fn empty_cert_store() -> X509Store {
        let mut builder = X509StoreBuilder::new().unwrap();
        builder
            .set_flags(X509VerifyFlags::PARTIAL_CHAIN | X509VerifyFlags::NO_CHECK_TIME)
            .unwrap();
        builder.set_purpose(X509PurposeId::ANY).unwrap();
        builder.build()
    }

    fn cert_store(&self, kind: EfiCmsCert) -> X509Store {
        let mut builder = X509StoreBuilder::new().unwrap();
        builder
            .set_flags(X509VerifyFlags::PARTIAL_CHAIN | X509VerifyFlags::NO_CHECK_TIME)
            .unwrap();
        builder.set_purpose(X509PurposeId::ANY).unwrap();
        for der in self
            .certs
            .iter()
            .filter(|(k, _cert)| *k == kind)
            .map(|(_k, cert)| cert)
        {
            let cert = X509::from_der(der).unwrap();
            builder.add_cert(cert).unwrap();
        }
        builder.build()
    }

    fn log_errors(stack: &openssl::error::ErrorStack) {
        for error in stack.errors() {
            debug!("openssl/verify: {error}");
        }
    }

    fn has_error(stack: &openssl::error::ErrorStack, find_code: u64) -> bool {
        stack
            .errors()
            .iter()
            .find(|e| e.code() == find_code)
            .is_some()
    }
}

impl EfiCms for EfiCmsOpenSSL {
    fn add_certs_vec(&mut self, kind: EfiCmsCert, certs: Vec<Vec<u8>>) {
        for c in certs {
            self.certs.push((kind, c));
        }
    }

    fn verify_signature(&self, signature: &[u8], data: Option<&[u8]>) -> Result<(), EfiCmsError> {
        let pkcs7 = Pkcs7::from_der(signature)?;
        let certs = Stack::new().unwrap();
        let empty = Self::empty_cert_store();

        debug!("openssl/verify: checking signature");
        match pkcs7.verify(
            &certs,
            &empty,
            data,
            None,
            Pkcs7Flags::BINARY | Pkcs7Flags::NOVERIFY,
        ) {
            Ok(_) => {
                debug!("openssl/verify: passed");
                Ok(())
            }
            Err(e) => {
                Self::log_errors(&e);
                Err(e.into())
            }
        }
    }

    fn verify_certificate(
        &self,
        signature: &[u8],
        data: Option<&[u8]>,
    ) -> Result<EfiCmsCert, EfiCmsError> {
        let pkcs7 = Pkcs7::from_der(signature)?;
        let certs = Stack::new().unwrap();

        for kind in [
            EfiCmsCert::PK,
            EfiCmsCert::KEK,
            EfiCmsCert::db,
            EfiCmsCert::MokList,
        ] {
            debug!("openssl/verify: checking {kind:?} certificates");
            let store = self.cert_store(kind);
            match pkcs7.verify(&certs, &store, data, None, Pkcs7Flags::BINARY) {
                Ok(_) => {
                    debug!("openssl/verify: passed");
                    return Ok(kind);
                }
                Err(e) => {
                    Self::log_errors(&e);
                    // - "unable to get local issuer certificate": continue, try next cert set
                    // - anyting else: return error
                    if !Self::has_error(&e, 0x10800075) {
                        return Err(e.into());
                    }
                }
            }
        }
        Err(EfiCmsError::CertificateNotTrusted)
    }
}
