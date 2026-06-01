use kdc::compose::{
    down::ComposeDownRequest, logs::ComposeLogRequest, restart::ComposeRestartRequest,
    up::ComposeUpRequest,
};

#[test]
fn compose_up_request_defaults() {
    let request = ComposeUpRequest::default();
    assert!(request.detached);
    assert!(request.services.is_empty());
    assert!(!request.build);
}

#[test]
fn compose_up_request_with_services() {
    let request = ComposeUpRequest {
        detached: true,
        services: vec!["web".to_string(), "db".to_string()],
        build: true,
    };
    assert_eq!(request.services.len(), 2);
    assert!(request.build);
}

#[test]
fn compose_down_request_defaults() {
    let request = ComposeDownRequest::default();
    assert!(!request.remove_volumes);
    assert!(request.remove_orphans);
}

#[test]
fn compose_down_with_volumes() {
    let request = ComposeDownRequest {
        remove_volumes: true,
        remove_orphans: true,
    };
    assert!(request.remove_volumes);
    assert!(request.remove_orphans);
}

#[test]
fn compose_restart_without_service() {
    let request = ComposeRestartRequest { service: None };
    assert!(request.service.is_none());
}

#[test]
fn compose_restart_specific_service() {
    let request = ComposeRestartRequest {
        service: Some("web".to_string()),
    };
    assert_eq!(request.service, Some("web".to_string()));
}

#[test]
fn compose_log_request_defaults() {
    let request = ComposeLogRequest::default();
    assert!(!request.follow);
    assert!(request.service.is_none());
    assert_eq!(request.tail, Some(100));
}

#[test]
fn compose_log_request_with_service() {
    let request = ComposeLogRequest {
        follow: false,
        service: Some("api".to_string()),
        tail: Some(50),
    };
    assert_eq!(request.service, Some("api".to_string()));
    assert_eq!(request.tail, Some(50));
}
