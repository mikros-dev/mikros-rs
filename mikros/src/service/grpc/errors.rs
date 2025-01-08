// Module internal error
crate::module_errors!(
    Error{
        TransportInitFailure(s: String) => "could not initialize transport layer: {}"
    }
);
