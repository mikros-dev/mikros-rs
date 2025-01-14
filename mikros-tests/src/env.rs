
#[cfg(test)]
mod tests {
    use mikros_macros::{Env, Lifecycle};

    #[derive(Env, Debug, Lifecycle, Clone)]
    struct Example {
        #[env(variable = "NAME", default = "John Doe")]
        name: String,

        #[env(variable = "AGE", default = "42")]
        age: i32,

        #[env(variable = "LIMIT", default = "0")]
        limit: i32,

        #[env(skip)]
        unused: bool,
    }

    #[test]
    fn test_derive_env() {
        let e = Example::from_env();
        println!("{:?}", e);

       let ee = Example::from_env_with_suffix("_service");
       println!("{:?}", ee);
    }
}