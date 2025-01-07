
// A macro to ease internal modules error creation with common behavior
// implemented. It creates an enum $name with all $entry defined. All
// errors created from this enum are returned as Internal for the client.
//
// The entries can have 0 or more values in addition to your description
// message.
//
// It can be called like this:
// internal_error!(
//      Error {
//          Internal(msg: String) => "Internal error {}",
//          NotFound => "Not found"
//      }
// )
//
// And an enum like this will be declared:
//
// enum Error {
//      Internal(String),
//      NotFound,
// }
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
