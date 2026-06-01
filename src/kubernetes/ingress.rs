#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngressSummary {
    pub name: String,
    pub host: String,
}

pub fn list(namespace: &str) -> anyhow::Result<Vec<IngressSummary>> {
    let output = crate::kubernetes::command::kubectl_list("ingress", Some(namespace))?;
    Ok(parse_ingress(&output))
}

pub fn get(name: &str, namespace: &str) -> anyhow::Result<String> {
    crate::kubernetes::command::run_kubectl(&[
        "get", "ingress", name, "-n", namespace, "-o", "yaml",
    ])
}

fn parse_ingress(output: &str) -> Vec<IngressSummary> {
    crate::kubernetes::command::non_empty_lines(output)
        .filter_map(|line| {
            let columns = line.split_whitespace().collect::<Vec<_>>();
            Some(IngressSummary {
                name: columns.first()?.to_string(),
                host: columns.get(2).unwrap_or(&"").to_string(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::parse_ingress;

    #[test]
    fn parses_ingress() {
        let ingress = parse_ingress("web nginx app.example.com 1.2.3.4 80 1m\n");
        assert_eq!(ingress[0].host, "app.example.com");
    }
}
