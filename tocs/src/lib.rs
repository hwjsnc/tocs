#![forbid(unsafe_code)]

mod io;

pub use io::{PaddedAsciiStr, PaddedAsciiString, PaddedByteStr, PaddedByteString, PaddedUtf8Str, PaddedUtf8String};

#[cfg(feature = "compression")]
pub mod compression;

pub mod bc7;
pub mod dds;

#[cfg(feature = "pkg")]
pub mod pka;
#[cfg(feature = "pkg")]
pub mod pkg;

pub mod tbl;

pub mod script;
