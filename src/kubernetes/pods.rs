#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PodSummary {
    pub name: String,
    pub namespace: String,
    pub status: String,
}

pub fn list(namespace: &str) -> anyhow::Result<Vec<PodSummary>> {
    let output = crate::kubernetes::command::kubectl_list("pods", Some(namespace))?;
    Ok(parse_pods(namespace, &output))
}

pub fn get(name: &str, namespace: &str) -> anyhow::Result<String> {
    crate::kubernetes::command::run_kubectl(&["get", "pod", name, "-n", namespace, "-o", "yaml"])
}

pub fn delete(name: &str, namespace: &str) -> anyhow::Result<String> {
    crate::kubernetes::command::run_kubectl(&["delete", "pod", name, "-n", namespace])
}

pub fn logs(name: &str, namespace: &str, tail: Option<u16>) -> anyhow::Result<Vec<String>> {
    let tail = tail.unwrap_or(100).to_string();
    let output =
        crate::kubernetes::command::run_kubectl(&["logs", name, "-n", namespace, "--tail", &tail])?;
    Ok(crate::kubernetes::command::non_empty_lines(&output)
        .map(str::to_string)
        .collect())
}

fn parse_pods(namespace: &str, output: &str) -> Vec<PodSummary> {
    crate::kubernetes::command::non_empty_lines(output)
        .filter_map(|line| {
            let columns = line.split_whitespace().collect::<Vec<_>>();
            Some(PodSummary {
                name: columns.first()?.to_string(),
                namespace: namespace.to_string(),
                status: columns.get(2).unwrap_or(&"Unknown").to_string(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::parse_pods;

    #[test]
    fn parses_pod_rows() {
        let pods = parse_pods("default", "web-abc 1/1 Running 0 1m\n");
        assert_eq!(pods[0].name, "web-abc");
        assert_eq!(pods[0].status, "Running");
    }
}
