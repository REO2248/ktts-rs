#![expect(
    unused_crate_dependencies,
    reason = "deps used by other targets of this package"
)]
pub mod codec;
#[cfg(feature = "embed")]
pub mod embedded;
pub mod pipeline;
pub mod types;
pub mod wav;
