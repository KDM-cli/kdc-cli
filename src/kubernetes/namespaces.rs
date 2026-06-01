#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespaceSummary {
    pub name: String,
}

pub fn list() -> anyhow::Result<Vec<NamespaceSummary>> {
    let output = crate::kubernetes::command::kubectl_list("namespaces", None)?;
    Ok(parse_namespaces(&output))
}

pub fn get(name: &str) -> anyhow::Result<String> {
    crate::kubernetes::command::run_kubectl(&["get", "namespace", name, "-o", "yaml"])
}

fn parse_namespaces(output: &str) -> Vec<NamespaceSummary> {
    crate::kubernetes::command::non_empty_lines(output)
        .filter_map(|line| {
            let name = line.split_whitespace().next()?;
            Some(NamespaceSummary {
                name: name.to_string(),
            })
        })
        .collect()
}
