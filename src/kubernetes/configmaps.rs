#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigMapSummary {
    pub name: String,
}

pub fn list(namespace: &str) -> anyhow::Result<Vec<ConfigMapSummary>> {
    let output = crate::kubernetes::command::kubectl_list("configmaps", Some(namespace))?;
    Ok(parse_names(&output))
}

pub fn get(name: &str, namespace: &str) -> anyhow::Result<String> {
    crate::kubernetes::command::run_kubectl(&[
        "get",
        "configmap",
        name,
        "-n",
        namespace,
        "-o",
        "yaml",
    ])
}

fn parse_names(output: &str) -> Vec<ConfigMapSummary> {
    crate::kubernetes::command::non_empty_lines(output)
        .filter_map(|line| {
            let name = line.split_whitespace().next()?;
            Some(ConfigMapSummary {
                name: name.to_string(),
            })
        })
        .collect()
}
