
// Module internal errors
crate::module_errors!(
    Error {
        SettingsError(e: String) => "{}",
        VariableNotSet(v: String) => "'{}' is not set"
    }
);