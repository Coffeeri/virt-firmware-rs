//! efi guid database

#![allow(non_upper_case_globals)]

use uuid::{uuid, Uuid};

pub const EfiGlobalVariable: Uuid = uuid!("8be4df61-93ca-11d2-aa0d-00e098032b8c");
pub const LoaderInfo: Uuid = uuid!("4a67b082-0a4c-41cf-b6c7-440b29bb8c4f");
pub const VMMBootOrder: Uuid = uuid!("668f4529-63d0-4bb5-b65d-6fbb9d36a44a");
