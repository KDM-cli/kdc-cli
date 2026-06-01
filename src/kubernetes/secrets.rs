#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretSummary {
    pub name: String,
}

pub fn list(namespace: &str) -> anyhow::Result<Vec<SecretSummary>> {
    let output = crate::kubernetes::command::kubectl_list("secrets", Some(namespace))?;
    Ok(parse_names(&output))
}

pub fn get(name: &str, namespace: &str) -> anyhow::Result<String> {
    crate::kubernetes::command::run_kubectl(&["get", "secret", name, "-n", namespace, "-o", "yaml"])
}

fn parse_names(output: &str) -> Vec<SecretSummary> {
    crate::kubernetes::command::non_empty_lines(output)
        .filter_map(|line| {
            let name = line.split_whitespace().next()?;
            Some(SecretSummary {
                name: name.to_string(),
            })
        })
        .collect()
}
