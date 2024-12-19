
// Module internal errors
crate::internal_errors!(
    Error {
        SettingsError(e: String) => "{}",
        VariableNotSet(v: String) => "'{}' is not set"
    }
);