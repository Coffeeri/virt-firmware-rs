use uefi::{CStr16, Error, Status};
use zerocopy::Ref;

pub fn cstr16_from_bytes_until_nul(bytes: &[u8]) -> Result<&CStr16, Error<&'static str>> {
    let words = Ref::<_, [u16]>::from_bytes(bytes)
        .or(Err(Error::new(Status::BAD_BUFFER_SIZE, "convert to u16")))?;
    let cstr16 = CStr16::from_u16_until_nul(Ref::into_ref(words)).or(Err(Error::new(
        Status::BAD_BUFFER_SIZE,
        "convert to cstr16",
    )))?;
    Ok(cstr16)
}

pub fn cstr16_from_bytes_with_nul(bytes: &[u8]) -> Result<&CStr16, Error<&'static str>> {
    let words = Ref::<_, [u16]>::from_bytes(bytes)
        .or(Err(Error::new(Status::BAD_BUFFER_SIZE, "convert to u16")))?;
    let cstr16 = CStr16::from_u16_with_nul(Ref::into_ref(words)).or(Err(Error::new(
        Status::BAD_BUFFER_SIZE,
        "convert to cstr16",
    )))?;
    Ok(cstr16)
}

pub fn u64_to_status(s: u64) -> Status {
    unsafe { core::mem::transmute(s) }
}
