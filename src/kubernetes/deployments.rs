#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeploymentSummary {
    pub name: String,
    pub namespace: String,
    pub ready: String,
}

pub fn list(namespace: &str) -> anyhow::Result<Vec<DeploymentSummary>> {
    let output = crate::kubernetes::command::kubectl_list("deployments", Some(namespace))?;
    Ok(parse_deployments(namespace, &output))
}

pub fn get(name: &str, namespace: &str) -> anyhow::Result<String> {
    crate::kubernetes::command::run_kubectl(&[
        "get",
        "deployment",
        name,
        "-n",
        namespace,
        "-o",
        "yaml",
    ])
}

pub fn scale(name: &str, namespace: &str, replicas: u16) -> anyhow::Result<String> {
    crate::kubernetes::command::run_kubectl(&[
        "scale",
        "deployment",
        name,
        "-n",
        namespace,
        "--replicas",
        &replicas.to_string(),
    ])
}

fn parse_deployments(namespace: &str, output: &str) -> Vec<DeploymentSummary> {
    crate::kubernetes::command::non_empty_lines(output)
        .filter_map(|line| {
            let columns = line.split_whitespace().collect::<Vec<_>>();
            Some(DeploymentSummary {
                name: columns.first()?.to_string(),
                namespace: namespace.to_string(),
                ready: columns.get(1).unwrap_or(&"unknown").to_string(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::parse_deployments;

    #[test]
    fn parses_deployment_rows() {
        let deployments = parse_deployments("default", "web 2/2 2 2 3m\n");
        assert_eq!(deployments[0].name, "web");
        assert_eq!(deployments[0].ready, "2/2");
    }
}
