/// The policy under which we will gzip the request body that is sent to the push notification servers
#[derive(Debug, Clone, Copy)]
pub enum GzipPolicy {
    /// Gzip only if the body is larger than the given number of bytes.
    /// The default is 1024 bytes.
    ZipGreaterThanTreshold(usize),

    /// Never Gzip the request body
    Never,

    /// Always Gzip the request body
    Always,
}

impl Default for GzipPolicy {
    fn default() -> Self {
        GzipPolicy::ZipGreaterThanTreshold(1024)
    }
}
