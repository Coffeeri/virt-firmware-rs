extern crate alloc;
use alloc::string::String;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct File {
    pub location: String,
}

#[derive(Deserialize, Debug)]
pub struct Disk {
    pub disk: File,
}

#[derive(Deserialize, Debug)]
pub struct Formats {
    pub iso: Disk,
}

#[derive(Deserialize, Debug)]
pub struct Artifact {
    pub release: String,
    pub formats: Formats,
}

#[derive(Deserialize, Debug)]
pub struct Artifacts {
    pub metal: Artifact,
}

#[derive(Deserialize, Debug)]
pub struct Architecture {
    pub artifacts: Artifacts,
}

#[derive(Deserialize, Debug)]
pub struct Architectures {
    pub aarch64: Architecture,
    pub x86_64: Architecture,
}

#[derive(Deserialize, Debug)]
pub struct Stream {
    pub architectures: Architectures,
}
