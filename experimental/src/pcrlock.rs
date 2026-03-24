use core::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use zerocopy::Ref;

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_camel_case_types)]
pub enum EventType {
    // generic
    EV_SEPARATOR,
    // pcr 0
    EV_S_CRTM_VERSION,
    EV_EFI_PLATFORM_FIRMWARE_BLOB,
    // pcr 7
    EV_EFI_VARIABLE_DRIVER_CONFIG,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventContent {
    pub event_type: EventType,
    pub event_data: String,
}

impl EventContent {
    fn data(&self) -> Option<Vec<u8>> {
        hex::decode(&self.event_data).ok()
    }

    fn parse_cstr16(b: &[u8]) -> Option<String> {
        let w = Ref::<_, [u16]>::from_bytes(b).ok()?;
        let s = String::from_utf16_lossy(Ref::into_ref(w));
        Some(s)
    }

    fn parse_fw_version(&self) -> Option<String> {
        let b = self.data()?;
        Self::parse_cstr16(&b)
    }

    fn parse_fw_blob(&self) -> Option<(u64, u64)> {
        let b = self.data()?;
        let u = Ref::<_, [u64]>::from_bytes(b.as_ref()).ok()?;
        let addr = u.first()?;
        let size = u.get(1)?;
        Some((*addr, *size))
    }
}

impl Display for EventContent {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        write!(f, "{:?}", self.event_type)?;
        match self.event_type {
            EventType::EV_SEPARATOR => {}
            EventType::EV_S_CRTM_VERSION => {
                if let Some(v) = self.parse_fw_version() {
                    write!(f, " version={v}")?;
                }
            }
            EventType::EV_EFI_PLATFORM_FIRMWARE_BLOB => {
                if let Some((a, s)) = self.parse_fw_blob() {
                    write!(f, " addr=0x{a:x} size=0x{s:x}")?;
                }
            }
            _ => {
                let dots = if self.event_data.len() > 8 { "..." } else { "" };
                write!(f, " bytes={:0.8}{}", self.event_data, dots)?
            }
        };
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_camel_case_types)]
pub enum HashAlg {
    // sha1,
    sha256,
    sha384,
    sha512,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventDigest {
    #[serde(rename = "hashAlg")]
    pub hash_alg: HashAlg,
    pub digest: String,
}

impl Display for EventDigest {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        let dots = if self.digest.len() > 8 { "..." } else { "" };
        write!(f, "{:?}={:0.8}{}", self.hash_alg, self.digest, dots)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventRecord {
    pub pcr: u8,
    pub digests: Vec<EventDigest>,
    pub content_type: String, // pcclient_std
    pub content: EventContent,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pcrlock {
    pub records: Vec<EventRecord>,
}
