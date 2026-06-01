#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KubernetesEvent {
    pub reason: String,
    pub message: String,
}

pub fn list(namespace: &str) -> anyhow::Result<Vec<KubernetesEvent>> {
    let output = crate::kubernetes::command::run_kubectl(&[
        "get",
        "events",
        "-n",
        namespace,
        "--no-headers",
    ])?;
    Ok(parse_events(&output))
}

fn parse_events(output: &str) -> Vec<KubernetesEvent> {
    crate::kubernetes::command::non_empty_lines(output)
        .filter_map(|line| {
            let columns = line.split_whitespace().collect::<Vec<_>>();
            let reason = columns.get(2).or_else(|| columns.first())?;
            Some(KubernetesEvent {
                reason: reason.to_string(),
                message: line.to_string(),
            })
        })
        .collect()
}
