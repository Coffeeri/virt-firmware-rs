extern crate alloc;
use alloc::vec::Vec;
use log::debug;

use openssl::cms::{CMSOptions, CmsContentInfo};
use openssl::x509::store::{X509Store, X509StoreBuilder};
use openssl::x509::verify::X509VerifyFlags;
use openssl::x509::{X509PurposeId, X509};

use crate::eficms::{EfiCms, EfiCmsCert, EfiCmsError};

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

    fn verify_signature(&self, signature: &[u8], data: &[u8]) -> Result<(), EfiCmsError> {
        let mut cms = CmsContentInfo::from_der(signature)?;

        debug!("openssl/verify: checking signature");
        match cms.verify(
            None,
            None,
            Some(data),
            None,
            CMSOptions::BINARY | CMSOptions::NO_SIGNER_CERT_VERIFY,
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

    fn verify_certificate(&self, signature: &[u8], data: &[u8]) -> Result<EfiCmsCert, EfiCmsError> {
        let mut cms = CmsContentInfo::from_der(signature)?;

        for kind in [
            EfiCmsCert::PK,
            EfiCmsCert::KEK,
            EfiCmsCert::db,
            EfiCmsCert::MokList,
        ] {
            debug!("openssl/verify: checking {kind:?} certificates");
            let store = self.cert_store(kind);
            match cms.verify(
                None,
                Some(&store),
                Some(data),
                None,
                CMSOptions::BINARY | CMSOptions::NO_CONTENT_VERIFY,
            ) {
                Ok(_) => {
                    debug!("openssl/verify: passed");
                    return Ok(kind);
                }
                Err(e) => {
                    Self::log_errors(&e);
                    // "unable to get local issuer certificate"
                    // -> continue, try next cert set
                    if Self::has_error(&e, 0x17000064) {
                        continue;
                    }
                    // anyting else -> return error
                    return Err(e.into());
                }
            }
        }
        Err(EfiCmsError::CertificateNotTrusted)
    }
}
