#[cfg(feature = "eficmsopenssl")]
pub mod openssl;
pub mod parse;

extern crate alloc;
use alloc::vec::Vec;

use crate::efivar::sigdb::EfiSigDB;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum EfiCmsCert {
    PK,
    KEK,
    db,
    MokList,
}

#[derive(Debug)]
pub enum EfiCmsError {
    UnspecifiedError,
    NotImplemented,
    UnknownAlgorithmCombination,
    ContentIsNoSignature,
    MissingSignerInfo,
    MissingIssuerAndSerialNumber,
    MissingCertificates,
    MissingIssuerCertificate,
    MissingSignedAttributes,
    MissingEmbeddedContent,
    CertificateNotTrusted,
    NoAuthenticodeSignature,
    Der(der::Error),
    #[cfg(feature = "eficmsopenssl")]
    OpenSSL(::openssl::error::ErrorStack),
}

impl core::fmt::Display for EfiCmsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnspecifiedError => write!(f, "unspecified error"),
            Self::NotImplemented => write!(f, "not implemented"),
            Self::UnknownAlgorithmCombination => write!(f, "unknown algorithm combination"),
            Self::ContentIsNoSignature => write!(f, "content is not a signature"),
            Self::MissingSignerInfo => write!(f, "missing signer info"),
            Self::MissingIssuerAndSerialNumber => write!(f, "missing issuer and serial number"),
            Self::MissingCertificates => write!(f, "missing certificates"),
            Self::MissingIssuerCertificate => write!(f, "missing issuer certificate"),
            Self::MissingSignedAttributes => write!(f, "missing signed attributes"),
            Self::MissingEmbeddedContent => write!(f, "missing embedded content"),
            Self::CertificateNotTrusted => write!(f, "certificate is not trusted"),
            Self::NoAuthenticodeSignature => write!(f, "no authenticode signature"),
            Self::Der(e) => write!(f, "der: {}", e),
            #[cfg(feature = "eficmsopenssl")]
            Self::OpenSSL(e) => write!(f, "openssl: {}", e),
        }
    }
}

impl core::error::Error for EfiCmsError {}

impl From<der::Error> for EfiCmsError {
    fn from(der: der::Error) -> Self {
        Self::Der(der)
    }
}

pub trait EfiCms {
    fn add_certs_vec(&mut self, kind: EfiCmsCert, cert: Vec<Vec<u8>>);

    fn add_certs_sigdb(&mut self, kind: EfiCmsCert, blob: &[u8]) {
        let Some(sigdb) = EfiSigDB::new_from_bytes(blob) else {
            return;
        };

        let certs = sigdb
            .siglists
            .iter()
            .filter_map(|sl| sl.get_x509())
            .map(|der| der.to_vec())
            .collect();
        self.add_certs_vec(kind, certs);
    }

    fn verify_signature(&self, signature: &[u8], data: &[u8]) -> Result<(), EfiCmsError>;
    fn verify_certificate(&self, signature: &[u8], data: &[u8]) -> Result<EfiCmsCert, EfiCmsError>;
}

// ---------------------------------------------------------------------
// dummy implementation

#[derive(Default)]
pub struct EfiCmsNull {}

impl EfiCmsNull {
    pub fn new() -> Self {
        Self {}
    }
}

impl EfiCms for EfiCmsNull {
    fn add_certs_vec(&mut self, _kind: EfiCmsCert, _certs: Vec<Vec<u8>>) {}

    fn verify_signature(&self, _signature: &[u8], _data: &[u8]) -> Result<(), EfiCmsError> {
        Err(EfiCmsError::NotImplemented)
    }

    fn verify_certificate(
        &self,
        _signature: &[u8],
        _data: &[u8],
    ) -> Result<EfiCmsCert, EfiCmsError> {
        Err(EfiCmsError::NotImplemented)
    }
}
