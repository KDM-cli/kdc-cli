use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use kdc::{
    domain::project::ProjectStack,
    project::{capabilities, detector},
    ui::menus::CapabilityMenuGenerator,
};

#[test]
fn detects_project_stack_and_capabilities() {
    let root = temp_project("kdc-detect");
    fs::write(root.join("Dockerfile"), "FROM node:20\n").unwrap();
    fs::write(root.join("docker-compose.yml"), "services: {}\n").unwrap();
    fs::write(root.join("deployment.yaml"), "apiVersion: apps/v1\n").unwrap();
    fs::write(root.join("package.json"), "{}\n").unwrap();

    let project = detector::detect(root.clone()).unwrap();
    let capabilities = capabilities::from_project(&project);

    assert_eq!(project.stack, ProjectStack::Node);
    assert!(capabilities.docker);
    assert!(capabilities.compose);
    assert!(capabilities.kubernetes);
    assert!(capabilities.monitoring);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn generated_menus_follow_capabilities() {
    let root = temp_project("kdc-menu");
    fs::write(root.join("Dockerfile"), "FROM scratch\n").unwrap();

    let project = detector::detect(root.clone()).unwrap();
    let capabilities = capabilities::from_project(&project);
    let menus = CapabilityMenuGenerator::generate(&capabilities, &Default::default());
    let labels = menus
        .iter()
        .map(|item| item.label.as_str())
        .collect::<Vec<_>>();

    assert!(labels.contains(&"Docker"));
    assert!(!labels.contains(&"Kubernetes"));
    assert!(!labels.contains(&"Deployments"));

    fs::remove_dir_all(root).unwrap();
}

fn temp_project(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("{prefix}-{unique}"));
    fs::create_dir_all(&root).unwrap();
    root
}
