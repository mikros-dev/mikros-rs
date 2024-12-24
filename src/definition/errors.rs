
// Module internal errors
crate::internal_errors!(
    Error{
        InvalidDefinitions(e: String) => "invalid service definitions: {}",
        DefinitionFileNotFound(e: String) => "definition file not found: {}",
        CouldNotLoadFile(e: String) => "could not load definitions file: {}",
        MalformedToml(e: String) => "malformed toml definitions: {}",
        ServiceNotFound(s: String) => "service definitions not found: {}",
        EmptyServiceType => "no service type was defined for service",
        CouldNotLoad(f: String, d: String) => "could not load definitions for '{}': {}"
    }
);