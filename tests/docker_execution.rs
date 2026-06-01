use kdc::docker::{
    build::BuildRequest, containers::ContainerSummary, images::DockerImage, logs::DockerLogLine,
    networks::DockerNetwork, run::RunRequest, volumes::DockerVolume,
};

#[test]
fn build_request_creates_full_tag() {
    let request = BuildRequest {
        image: "myapp".to_string(),
        tag: "latest".to_string(),
    };
    assert_eq!(request.full_tag(), "myapp:latest");
}

#[test]
fn build_request_with_registry_prefix() {
    let request = BuildRequest {
        image: "ghcr.io/org/myapp".to_string(),
        tag: "v2.0.0".to_string(),
    };
    assert_eq!(request.full_tag(), "ghcr.io/org/myapp:v2.0.0");
}

#[test]
fn run_request_default_is_detached() {
    let request = RunRequest::default();
    assert!(request.detached);
    assert!(request.ports.is_empty());
    assert!(request.env_vars.is_empty());
    assert!(request.name.is_none());
}

#[test]
fn run_request_with_all_fields() {
    let request = RunRequest {
        image: "nginx:alpine".to_string(),
        name: Some("web-server".to_string()),
        ports: vec!["8080:80".to_string(), "443:443".to_string()],
        env_vars: vec![
            ("NODE_ENV".to_string(), "production".to_string()),
            ("PORT".to_string(), "3000".to_string()),
        ],
        detached: true,
    };

    assert_eq!(request.image, "nginx:alpine");
    assert_eq!(request.name, Some("web-server".to_string()));
    assert_eq!(request.ports.len(), 2);
    assert_eq!(request.env_vars.len(), 2);
}

#[test]
fn container_summary_has_all_fields() {
    let container = ContainerSummary {
        id: "abc123def456".to_string(),
        name: "my-app".to_string(),
        image: "nginx:latest".to_string(),
        status: "Up 5 minutes".to_string(),
        ports: "0.0.0.0:80->80/tcp".to_string(),
    };

    assert_eq!(container.id, "abc123def456");
    assert_eq!(container.name, "my-app");
    assert_eq!(container.image, "nginx:latest");
    assert!(container.status.contains("Up"));
}

#[test]
fn docker_image_full_name_with_tag() {
    let image = DockerImage {
        repository: "myapp".to_string(),
        tag: "v1.0".to_string(),
        image_id: "sha256:abc123".to_string(),
        size: "150MB".to_string(),
    };
    assert_eq!(image.full_name(), "myapp:v1.0");
}

#[test]
fn docker_image_full_name_without_tag() {
    let image = DockerImage {
        repository: "myapp".to_string(),
        tag: "<none>".to_string(),
        image_id: "sha256:abc123".to_string(),
        size: "150MB".to_string(),
    };
    assert_eq!(image.full_name(), "myapp");
}

#[test]
fn docker_log_line_message() {
    let line = DockerLogLine {
        message: "INFO: Server started on port 8080".to_string(),
    };
    assert!(line.message.contains("8080"));
}

#[test]
fn docker_network_fields() {
    let network = DockerNetwork {
        id: "net123".to_string(),
        name: "app-network".to_string(),
        driver: "bridge".to_string(),
        scope: "local".to_string(),
    };
    assert_eq!(network.name, "app-network");
    assert_eq!(network.driver, "bridge");
}

#[test]
fn docker_volume_fields() {
    let volume = DockerVolume {
        name: "db-data".to_string(),
        driver: "local".to_string(),
        mountpoint: "/var/lib/docker/volumes/db-data/_data".to_string(),
    };
    assert_eq!(volume.name, "db-data");
    assert!(volume.mountpoint.contains("db-data"));
}
