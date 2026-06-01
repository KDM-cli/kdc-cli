use std::fs::{self, File};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static MOCK_BIN_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn setup_mock_bin() -> PathBuf {
    MOCK_BIN_DIR.get_or_init(|| {
        let temp = std::env::temp_dir().join(format!(
            "kdc-mock-bin-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&temp).unwrap();

        let kdc_home = temp.join(".kdc");
        fs::create_dir_all(&kdc_home).unwrap();
        std::env::set_var("KDC_HOME", &kdc_home);

        write_docker_script(&temp);
        write_kubectl_script(&temp);

        temp
    }).clone()
}

fn write_docker_script(temp: &Path) {
    let docker_path = temp.join("docker");
    let mut docker_file = File::create(&docker_path).unwrap();
    let docker_script = r#"#!/bin/bash
case "$1" in
  ps)
    if [[ "$*" == *"-a"* ]]; then
      echo -e "container123\tweb-app\tnginx:latest\tUp 5 minutes\t0.0.0.0:80->80/tcp"
    else
      echo -e "container123\tweb-app\tnginx:latest\tUp 5 minutes\t0.0.0.0:80->80/tcp"
    fi
    ;;
  logs)
    echo "line1"
    echo "line2"
    echo "line3"
    ;;
  volume)
    if [ "$2" = "ls" ]; then
      echo -e "db-data\tlocal\t/var/lib/docker/volumes/db-data/_data"
    fi
    ;;
  network)
    if [ "$2" = "ls" ]; then
      echo -e "net123\tapp-network\tbridge\tlocal"
    fi
    ;;
  images)
    echo -e "myapp\tlatest\tsha256:abc123\t150MB"
    ;;
  build)
    echo "Building image..."
    echo "Successfully built sha256:abc123"
    ;;
  run)
    echo "container123"
    ;;
  stop)
    echo "container123"
    ;;
  restart)
    echo "container123"
    ;;
  compose)
    case "$2" in
      logs)
        echo "compose log line 1"
        echo "compose log line 2"
        ;;
      config)
        echo "service1"
        echo "service2"
        ;;
      ps)
        echo "service1"
        ;;
      up)
        echo "Creating network..."
        echo "Starting container..."
        ;;
      down)
        echo "Stopping container..."
        echo "Removing network..."
        ;;
      *)
        ;;
    esac
    ;;
  info)
    echo "Docker Info mock"
    ;;
  inspect)
    echo "manifest-info-json"
    ;;
  manifest)
    if [ "$2" = "inspect" ]; then
      echo "manifest-info-json"
    fi
    ;;
  --version)
    echo "Docker version 24.0.7, build afdd53b"
    ;;
  *)
    echo "Unknown mock docker command: $*" >&2
    exit 0
    ;;
esac
"#;
    docker_file.write_all(docker_script.as_bytes()).unwrap();
    let mut perms = fs::metadata(&docker_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&docker_path, perms).unwrap();
}

fn write_kubectl_script(temp: &Path) {
    let kubectl_path = temp.join("kubectl");
    let mut kubectl_file = File::create(&kubectl_path).unwrap();
    let kubectl_script = r#"#!/bin/bash
case "$1" in
  cluster-info)
    echo "Kubernetes control plane is running at https://127.0.0.1:6443"
    ;;
  config)
    if [ "$2" = "current-context" ]; then
      echo "minikube"
    fi
    ;;
  get)
    if [ "$2" = "nodes" ]; then
      echo "node1 node2"
    fi
    ;;
  rollout)
    case "$2" in
      undo)
        echo "deployment.apps/deployment rolled back"
        ;;
      history)
        echo "REVISION  CHANGE-CAUSE"
        echo "1         Initial deployment"
        echo "2         Updated image"
        ;;
      status)
        echo "deployment \"my-app\" successfully rolled out"
        ;;
      *)
        ;;
    esac
    ;;
  apply)
    echo "deployment.apps/my-app configured"
    ;;
  version)
    echo "Client Version: v1.28.2"
    ;;
  *)
    echo "Unknown mock kubectl command: $*" >&2
    exit 0
    ;;
esac
"#;
    kubectl_file.write_all(kubectl_script.as_bytes()).unwrap();
    let mut perms = fs::metadata(&kubectl_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&kubectl_path, perms).unwrap();
}

pub fn set_mock_path() {
    let mock_bin = setup_mock_bin();
    let path = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("PATH", format!("{}:{}", mock_bin.to_string_lossy(), path));
}
