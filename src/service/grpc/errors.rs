
// Module internal error
crate::internal_errors!(
    Error{
        TransportInitFailure(s: String) => "could not initialize transport layer: {}"
    }
);