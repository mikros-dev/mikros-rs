/// Defines a custom error enum with standardized internal error handling.
///
/// This macro simplifies the creation of internal module-specific error types by:
///
/// - Declaring an enum with the given variants and associated data.
/// - Providing a `description()` method that formats each variant into a human-readable message.
/// - Implementing `Debug` by printing the formatted description.
/// - Automatically converting the enum into a `mikros::errors::Error::Internal`
///   using the formatted description.
///
/// All generated errors are treated as internal errors (`Error::Internal`) when
/// converted, making this macro ideal for use inside internal features where
/// external error visibility is restricted.
///
/// # Syntax
///
/// ```rust,ignore
/// module_errors!(
///     ErrorName {
///         Variant1(arg1: Type1, arg2: Type2) => "Error occurred: {}, {}",
///         Variant2 => "A simple error message",
///         Variant3(msg: String) => "Something failed: {}"
///     }
/// );
/// ```
///
/// # Example
///
/// ```rust,ignore
/// module_errors!(
///     MyError {
///         Internal(msg: String) => "Internal error: {}",
///         NotFound => "Resource not found"
///     }
/// );
///
/// fn do_something() -> Result<(), mikros::errors::Error> {
///     Err(MyError::Internal("disk full".into()).into())
/// }
/// ```
///
/// This expands to:
/// ```rust,ignore
/// pub enum MyError {
///     Internal(String),
///     NotFound,
/// }
///
/// impl MyError {
///     pub fn description(&self) -> String {
///         match self {
///             MyError::Internal(msg) => format!("Internal error: {}", msg),
///             MyError::NotFound => "Resource not found".to_string(),
///         }
///     }
/// }
///
/// impl Debug for MyError {
///     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
///         write!(f, "{}", self.description())
///     }
/// }
///
/// impl From<MyError> for mikros::errors::Error {
///     fn from(e: MyError) -> mikros::errors::Error {
///         mikros::errors::Error::Internal(e.description())
///     }
/// }
/// ```
#[macro_export]
macro_rules! module_errors {
      ($name:ident { $($entry:ident$(($($arg:ident : $arg_type:ty),*))? => $desc:expr),* }) => {
        pub enum $name {
            $(
                $entry$(($($arg_type),*))?,
            )*
        }

        impl $name {
            pub fn description(&self) -> String {
                match self {
                    $(
                        $name::$entry$(($($arg),*))? => format!($desc, $($($arg),*)?),
                    )*
                }
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.description())
            }
        }

        impl From<$name> for $crate::errors::Error {
            fn from(e: $name) -> $crate::errors::Error {
                $crate::errors::Error::Internal(e.description())
            }
        }
    };
}
