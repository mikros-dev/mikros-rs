
// Module internal errors
crate::internal_errors!(
    Error {
        InitFailure(e: String) => "could not initialize HTTP server: {}",
        ShutdownFailure(e: String) => "could not shutdown HTTP server: {}"
    }
);