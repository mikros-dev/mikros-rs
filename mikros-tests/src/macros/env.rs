
// Be aware that the tests here manipulate environment variables. Thus, they
// can affect behavior of other tests. We try to keep the variable names only
// for this module to avoid this.
#[cfg(test)]
mod tests {
    use mikros_macros::Env;

    #[test]
    fn test_struct_load_from_env() {
        #[derive(Env, Debug)]
        struct Example {
            #[env(variable = "TEST_1_NAME", default = "John Doe")]
            name: String,

            #[env(variable = "TEST_1_AGE", default = "42")]
            age: i32,

            #[env(variable = "TEST_1_LIMIT", default = "0")]
            limit: i32,

            #[env(variable = "TEST_BAR", default = "true")]
            bar: bool,

            #[env(variable = "TEST_FOO", default = "None")]
            foo: Option<i32>,

            #[env(variable = "TEST_FOO2", default = "None")]
            foo2: Option<i32>,

            #[env(variable = "TEST_FOO3", default = "42")]
            foo3: Option<i32>,

            #[env(variable = "TEST_FOO4", default = "true")]
            foo4: Option<bool>,

            unused: bool,
            data: std::collections::HashMap<String, String>,
        }

        std::env::set_var("TEST_1_NAME", "New Name");
        std::env::set_var("TEST_1_AGE", "84");
        std::env::set_var("TEST_1_LIMIT", "100");

        let e = Example::from_env();
        assert_eq!(e.name, "New Name");
        assert_eq!(e.age, 84);
        assert_eq!(e.limit, 100);
        assert_eq!(e.unused, false);
        assert_eq!(e.foo, None);
        assert_eq!(e.foo2, None);
        assert_eq!(e.data.len(), 0);
        assert_eq!(e.bar, true);
        assert_eq!(e.foo3, Some(42));
        assert_eq!(e.foo4, Some(true));
    }

    #[test]
    fn test_struct_load_from_env_with_suffix() {
        #[derive(Env, Debug)]
        #[env(suffix_delimiter = "_")]
        struct Example {
            #[env(variable = "TEST_2_NAME", default = "John Doe")]
            name: String,

            #[env(variable = "TEST_2_AGE", default = "42")]
            age: i32,

            #[env(variable = "TEST_2_LIMIT", default = "0")]
            limit: i32,

            #[env(variable = "TEST_2_BAR", default = "true")]
            bar: bool,

            unused: bool,
        }

        std::env::set_var("TEST_2_NAME_dev", "New Name 2");
        std::env::set_var("TEST_2_AGE_dev", "841");
        std::env::set_var("TEST_2_LIMIT_dev", "1001");

        let e = Example::from_env_with_suffix("dev", true);
        assert_eq!(e.name, "New Name 2");
        assert_eq!(e.age, 841);
        assert_eq!(e.limit, 1001);
        assert_eq!(e.unused, false);
        assert_eq!(e.bar, true);
    }

    #[test]
    fn test_struct_with_default_values() {
        #[derive(Env, Debug)]
        struct Example {
            #[env(variable = "TEST_3_NAME", default = "John Doe")]
            name: String,

            #[env(variable = "TEST_3_AGE", default = "42")]
            age: i32,

            #[env(variable = "TEST_3_LIMIT", default = "0")]
            limit: i32,

            unused: bool,
        }

        let e = Example::from_env();
        assert_eq!(e.name, "John Doe");
        assert_eq!(e.age, 42);
        assert_eq!(e.limit, 0);
        assert_eq!(e.unused, false);
    }

    #[test]
    fn test_struct_without_env_attributes() {
        #[derive(Env, Debug)]
        struct Example {
            name: String,
            age: i32,
            limit: i32,
            unused: bool,
        }

        let e = Example::from_env();
        assert_eq!(e.name, "");
        assert_eq!(e.age, 0);
        assert_eq!(e.limit, 0);
        assert_eq!(e.unused, false);
    }
}