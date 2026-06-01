use crate::project::environment::Environment;

pub use crate::project::environment::Environment as DeployEnvironment;

/// Map an environment to a Kubernetes namespace.
pub fn resolve_namespace(env: &Environment) -> String {
    match env {
        Environment::Development => "default".to_string(),
        Environment::Staging => "staging".to_string(),
        Environment::Production => "production".to_string(),
    }
}

/// Parse an environment string into an `Environment` enum.
pub fn from_string(s: &str) -> Environment {
    match s.to_lowercase().as_str() {
        "staging" | "stg" => Environment::Staging,
        "production" | "prod" => Environment::Production,
        _ => Environment::Development,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn development_resolves_to_default_namespace() {
        assert_eq!(resolve_namespace(&Environment::Development), "default");
    }

    #[test]
    fn staging_resolves_to_staging_namespace() {
        assert_eq!(resolve_namespace(&Environment::Staging), "staging");
    }

    #[test]
    fn production_resolves_to_production_namespace() {
        assert_eq!(resolve_namespace(&Environment::Production), "production");
    }

    #[test]
    fn from_string_parses_variations() {
        assert_eq!(from_string("staging"), Environment::Staging);
        assert_eq!(from_string("stg"), Environment::Staging);
        assert_eq!(from_string("production"), Environment::Production);
        assert_eq!(from_string("prod"), Environment::Production);
        assert_eq!(from_string("development"), Environment::Development);
        assert_eq!(from_string("anything"), Environment::Development);
    }
}
