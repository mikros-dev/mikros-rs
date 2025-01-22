#[cfg(test)]
mod tests {
    use mikros::service::lifecycle::Lifecycle;
    use mikros::tokio;
    use mikros_macros::Lifecycle;

    #[tokio::test]
    async fn test_lifecycle_trait_ok() {
        #[allow(dead_code)]
        #[derive(Lifecycle, Clone)]
        struct Example {
            name: String,
        }

        let example = Example { name: "Example 1".to_string() };
        let result = example.on_finish().await.unwrap();
        assert_eq!(result, ());
    }
}