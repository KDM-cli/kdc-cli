#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceSummary {
    pub name: String,
    pub namespace: String,
}

pub fn list(namespace: &str) -> anyhow::Result<Vec<ServiceSummary>> {
    let output = crate::kubernetes::command::kubectl_list("services", Some(namespace))?;
    Ok(parse_services(namespace, &output))
}

pub fn get(name: &str, namespace: &str) -> anyhow::Result<String> {
    crate::kubernetes::command::run_kubectl(&[
        "get", "service", name, "-n", namespace, "-o", "yaml",
    ])
}

fn parse_services(namespace: &str, output: &str) -> Vec<ServiceSummary> {
    crate::kubernetes::command::non_empty_lines(output)
        .filter_map(|line| {
            let name = line.split_whitespace().next()?;
            Some(ServiceSummary {
                name: name.to_string(),
                namespace: namespace.to_string(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::parse_services;

    #[test]
    fn parses_services() {
        let services = parse_services("default", "web ClusterIP 10.0.0.1 <none> 80/TCP 1m\n");
        assert_eq!(services[0].name, "web");
    }
}
