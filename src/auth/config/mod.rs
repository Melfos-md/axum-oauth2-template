pub mod oauth;

#[derive(PartialEq, Clone)]

pub enum Environment {
    Development,
    Production,
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Environment::Production => write!(f, "prod"),
            Environment::Development => write!(f, "dev"),
        }
    }
}
