use cms::cert::{CertificateChoices, IssuerAndSerialNumber};
use cms::content_info::ContentInfo;
use cms::signed_data::{SignedData, SignerIdentifier, SignerInfo};
use der::asn1::{AnyRef, ObjectIdentifier, PrintableString};
use der::{Decode, Reader, SliceReader, Tag, Tagged};
use x509_cert::Certificate;

use virtfw_libefi::sb::certs::oids::{oid_to_name, OID_CMS_SIGNED_DATA};

use super::EfiCmsError;

pub fn asn1_dump(bytes: &[u8], depth: usize, max: usize) -> der::Result<()> {
    let mut reader = der::SliceReader::new(bytes)?;
    while !reader.is_finished() {
        let any = AnyRef::decode(&mut reader)?;
        let indent = "  ".repeat(depth);

        if any.tag().is_constructed() {
            println!("{indent}- {}: {} bytes", any.tag(), any.value().len());
            if depth < max {
                asn1_dump(any.value(), depth + 1, max)?;
            }
        } else if any.tag() == Tag::ObjectIdentifier {
            let oid = ObjectIdentifier::from_bytes(any.value())?;
            println!("{indent}- {}: {} ({oid})", any.tag(), oid_to_name(&oid));
        } else if any.tag() == Tag::PrintableString {
            let s = PrintableString::new(any.value())?;
            println!("{indent}- {}: {s}", any.tag());
        } else if any.tag() == Tag::OctetString || any.tag() == Tag::BitString {
            println!("{indent}- {}: {} bytes", any.tag(), any.value().len());
        } else {
            println!("{indent}- {}: {:02x?}", any.tag(), any.value());
        }
    }
    Ok(())
}

pub fn get_signer_info(sd: &SignedData) -> Result<&SignerInfo, EfiCmsError> {
    let Some(si) = sd.signer_infos.0.iter().next() else {
        return Err(EfiCmsError::MissingSignerInfo);
    };
    Ok(si)
}

pub fn get_issuer_serial(si: &SignerInfo) -> Result<&IssuerAndSerialNumber, EfiCmsError> {
    let SignerIdentifier::IssuerAndSerialNumber(sid) = &si.sid else {
        return Err(EfiCmsError::MissingIssuerAndSerialNumber);
    };
    Ok(sid)
}

pub fn get_certificate(cc: &CertificateChoices) -> Option<&Certificate> {
    if let CertificateChoices::Certificate(cert) = cc {
        Some(cert)
    } else {
        None
    }
}

pub fn get_issuer_cert(sd: &SignedData) -> Result<&Certificate, EfiCmsError> {
    let si = get_signer_info(sd)?;
    let sid = get_issuer_serial(si)?;

    let Some(ref certs) = sd.certificates else {
        return Err(EfiCmsError::MissingCertificates);
    };

    let Some(cert) = certs
        .0
        .iter()
        .filter_map(get_certificate)
        .filter(|c| c.tbs_certificate.issuer == sid.issuer)
        .find(|c| c.tbs_certificate.serial_number == sid.serial_number)
    else {
        return Err(EfiCmsError::MissingIssuerCertificate);
    };

    Ok(cert)
}

pub fn get_signed_data(signature: &[u8]) -> Result<SignedData, EfiCmsError> {
    // Construct a reader manually here rather than using
    // `Decode::from_der`, because there may be unused trailing data
    // in `bytes`, which causes a `TrailingData` error.
    let mut reader = SliceReader::new(signature)?;
    let ci = ContentInfo::decode(&mut reader)?;

    if ci.content_type != OID_CMS_SIGNED_DATA {
        return Err(EfiCmsError::ContentIsNoSignature);
    }
    let sd: SignedData = ci.content.decode_as()?;

    Ok(sd)
}
