use std::ops::Deref;

#[derive(Debug, Clone, serde_derive::Deserialize)]
pub struct ServiceName(String);

impl ServiceName {
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }

    pub fn to_snake_case(&self) -> String {
        let mut result = String::new();
        let mut prev_char: Option<char> = None;

        for (i, c) in self.0.char_indices() {
            if i > 0 && c.is_uppercase() {
                if let Some(prev) = prev_char {
                    if prev != '_' && prev != '-' {
                        result.push('_');
                    }
                }
            }

            if c == '-' {
                result.push('_');
            } else {
                result.push(c.to_ascii_lowercase());
            }

            prev_char = Some(c);
        }

        result
    }
}

impl Deref for ServiceName {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
