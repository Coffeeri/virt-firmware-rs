use cms::signed_data::SignerInfo;
use der::{Decode, Encode};
use x509_cert::spki::AlgorithmIdentifierOwned;
use x509_cert::Certificate;

use super::parse::{get_issuer_cert, get_signed_data, get_signer_info};
use super::{EfiCms, EfiCmsCert, EfiCmsError};

use virtfw_libefi::sb::certs::oids::*;

impl From<x509_verify::Error> for EfiCmsError {
    fn from(xv: x509_verify::Error) -> Self {
        Self::X509Verify(xv)
    }
}

// ---------------------------------------------------------------------
// RustCrypto variant, using x509_verify crate

#[derive(Default)]
pub struct EfiCmsNative {
    certs: Vec<(EfiCmsCert, Certificate)>,
}

impl EfiCmsNative {
    pub fn new() -> Self {
        Self { certs: Vec::new() }
    }

    fn map_algo(sinfo: &SignerInfo) -> Result<AlgorithmIdentifierOwned, EfiCmsError> {
        let d = sinfo.digest_alg.oid;
        let s = sinfo.signature_algorithm.oid;
        let combined = match (d, s) {
            (OID_ALGO_SHA256, OID_ALGO_RSA) => OID_ALGO_SHA256_RSA,
            (_, _) => {
                return Err(EfiCmsError::UnknownAlgorithmCombination);
            }
        };
        let algo = AlgorithmIdentifierOwned {
            oid: combined,
            parameters: None,
        };
        Ok(algo)
    }

    fn verify_signature(
        sinfo: &SignerInfo,
        data: Option<&[u8]>,
        cert: &Certificate,
    ) -> Result<(), EfiCmsError> {
        let message = if let Some(msg) = data {
            msg
        } else {
            let Some(sattr) = sinfo.signed_attrs.as_ref() else {
                return Err(EfiCmsError::MissingSignedAttributes);
            };
            &sattr.to_der()?
        };

        let spki = cert.tbs_certificate.subject_public_key_info.clone();
        let algo = Self::map_algo(sinfo)?;
        let signature = x509_verify::Signature::new(&algo, sinfo.signature.as_bytes());
        let vinfo = x509_verify::VerifyInfo::new(message.into(), signature);
        let vkey: x509_verify::VerifyingKey = spki.try_into()?;

        vkey.verify(&vinfo)?;
        Ok(())
    }

    fn verify_cert_one(cert: &Certificate, issuer: &Certificate) -> bool {
        let Ok(key) = x509_verify::VerifyingKey::try_from(issuer) else {
            return false;
        };
        key.verify(cert).is_ok()
    }

    fn verify_cert_trust(&self, cert: &Certificate) -> Option<EfiCmsCert> {
        for (kind, issuer) in &self.certs {
            if cert.tbs_certificate.issuer != issuer.tbs_certificate.subject {
                continue;
            }
            if Self::verify_cert_one(cert, issuer) {
                return Some(*kind);
            }
        }
        None
    }
}

impl EfiCms for EfiCmsNative {
    fn add_certs_vec(&mut self, kind: EfiCmsCert, certs: Vec<Vec<u8>>) {
        for c in certs.iter().filter_map(|c| Certificate::from_der(c).ok()) {
            self.certs.push((kind, c));
        }
    }

    fn verify_signature(&self, signature: &[u8], data: Option<&[u8]>) -> Result<(), EfiCmsError> {
        let sd = get_signed_data(signature)?;
        let si = get_signer_info(&sd)?;
        let cert = get_issuer_cert(&sd)?;

        Self::verify_signature(si, data, cert)?;
        Ok(())
    }

    fn verify_certificate(
        &self,
        signature: &[u8],
        _data: Option<&[u8]>,
    ) -> Result<EfiCmsCert, EfiCmsError> {
        let sd = get_signed_data(signature)?;
        let cert = get_issuer_cert(&sd)?;

        match self.verify_cert_trust(cert) {
            Some(c) => Ok(c),
            None => Err(EfiCmsError::CertificateNotTrusted),
        }
    }
}
