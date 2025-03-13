use crate::auth::ssh::schema::SSH;
use crate::cluster::schema::{Cluster, ClusterInit};
use anyhow::{anyhow, Context};
use regex::Regex;
use ssh2::Session;
use std::fs;
use std::path::Path;
use std::process::Stdio;

pub(crate) mod command;
pub(crate) mod schema;

impl Cluster {
  pub fn generate_json_file_from_node(path: &Path) -> anyhow::Result<()> {
    std::process::Command::new("node")
      .arg(path)
      .stdout(Stdio::inherit())
      .stderr(Stdio::inherit())
      .output()
      .context("Failed to read doseid config")?;
    Ok(())
  }

  pub fn from_json_file() -> anyhow::Result<Self> {
    let cluster_path = Path::new(".").join(".dosei/cluster.json");
    let app_data = fs::read_to_string(&cluster_path)
      .context(format!("Failed to read cluster file at {:?}", cluster_path))?;
    Ok(serde_json::from_str::<Self>(&app_data)?)
  }
  fn validate_domain(&self) -> bool {
    // This regex checks for a valid domain name pattern
    // Domain must have at least one dot and valid characters
    let domain_regex =
      Regex::new(r"^([a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?\.)+[a-zA-Z]{2,}$").unwrap();
    domain_regex.is_match(&self.name)
  }
}

const REMOTE_CLUSTER_DEPLOY_LOCK_FILE: &str = "~/.dosei/deploy.lock";

impl ClusterInit {
  pub fn create_lock(&self, session: &Session) -> anyhow::Result<()> {
    // First ensure ~/.dosei directory exists
    SSH::execute_command(session, "mkdir -p ~/.dosei")?;

    // Check if lock file exists
    let remote_lock = SSH::check_file_exists(session, REMOTE_CLUSTER_DEPLOY_LOCK_FILE)?;
    if remote_lock {
      return Err(anyhow!("Another cluster deployment is in progress. If you're sure no other process is running, remove ~/.dosei/deploy.lock manually."));
    }
    // Create the lock file
    SSH::execute_command(
      session,
      &format!("touch {}", REMOTE_CLUSTER_DEPLOY_LOCK_FILE),
    )?;

    Ok(())
  }

  pub fn run_doseid_container(&self, session: &Session) -> anyhow::Result<()> {
    // First ensure ~/.dosei directory exists
    SSH::execute_command(session, "mkdir -p ~/.dosei")?;

    // Define container name
    let container_name = "doseid";

    // Check if a container with this name is already running using grep and wc
    let check_running_command = format!(
      "if [ $(docker ps -q -f name={} | wc -l) -gt 0 ]; then exit 1; else exit 0; fi",
      container_name
    );

    let running_result = SSH::execute_command(session, &check_running_command);
    if running_result.is_err() {
      return Err(anyhow!(
        "A container named '{}' is already running. Stop or remove it first.",
        container_name
      ));
    }

    // Check if the container exists but is stopped
    let check_exists_command = format!(
      "if [ $(docker ps -a -q -f name={} | wc -l) -gt 0 ]; then exit 0; else exit 1; fi",
      container_name
    );

    let exists_result = SSH::execute_command(session, &check_exists_command);
    if exists_result.is_ok() {
      // Container exists, remove it
      SSH::execute_command(session, &format!("docker rm {}", container_name))?;
    }

    // Get the current package version for the docker image tag
    let image_version = env!("CARGO_PKG_VERSION");
    let docker_image = format!("doseiai/test:{}", image_version);

    // Run the docker container with volume mounting from ~/.dosei to /var/lib/dosei
    // and assign the container name
    let docker_command = format!(
      "docker run -d \
      -p 443:443 -p 80:80 \
      -v /var/run/docker.sock:/var/run/docker.sock \
      -v ~/.dosei/doseid:/var/lib/doseid \
      -v ~/.dosei/postgres:/var/lib/postgresql/data \
      --name {} {}
      ",
      container_name, docker_image
    );

    // Execute the docker run command
    let x = SSH::execute_command(session, &docker_command)?;
    println!("{:?}", x);

    Ok(())
  }

  pub fn remove_lock(&self, session: &Session) -> anyhow::Result<()> {
    SSH::execute_command(
      session,
      &format!("rm -f {}", REMOTE_CLUSTER_DEPLOY_LOCK_FILE),
    )?;
    Ok(())
  }

  fn install_docker_on_remote(&self, session: &Session, username: &str) -> anyhow::Result<()> {
    println!("\n🐳 Docker Check:");
    // Check for docker binary
    let docker_exists = SSH::check_file_exists(&session, "/usr/bin/docker")?
      || SSH::check_file_exists(&session, "/bin/docker")?;

    if !docker_exists {
      println!("❌ Docker is NOT installed on the remote server");

      println!("\n🚀 Installing Docker...");
      if let Err(e) = self._install_docker(&session, username) {
        println!("❌ Failed to install Docker: {}", e);
        return Err(e);
      }
      println!("✅ Docker successfully installed!");
    } else {
      // Try to get Docker version
      let (exit_code, docker_version) = SSH::execute_command(&session, "docker --version")?;

      if exit_code == 0 && !docker_version.is_empty() {
        println!("✅ Docker is installed on the remote server:");
        println!("🐳 {}", docker_version.trim());
      } else {
        println!("⚠️ Docker binary found but failed to get version information");

        println!("\n🔄 Reinstalling Docker...");
        if let Err(e) = self._install_docker(&session, username) {
          println!("❌ Failed to reinstall Docker: {}", e);
          return Err(e);
        }
        println!("✅ Docker successfully reinstalled!");
      }
    }
    Ok(())
  }

  fn _install_docker(&self, session: &Session, username: &str) -> anyhow::Result<()> {
    // Step 1: Remove potentially conflicting packages
    println!("Removing conflicting packages...");
    let remove_cmd = "sudo apt-get remove -y docker.io docker-doc docker-compose docker-compose-v2 podman-docker containerd runc";
    let (exit_code, output) = SSH::execute_command(session, remove_cmd)?;
    if exit_code != 0 {
      println!("⚠️ Warning during package removal: {}", output);
      // Continue anyway as these might not be installed
    }

    // Step 2: Update package index and install prerequisites
    println!("Installing prerequisites...");
    SSH::execute_command(session, "sudo apt-get update")?;
    SSH::execute_command(session, "sudo apt-get install -y ca-certificates curl")?;

    // Step 3: Set up Docker's GPG key
    println!("Setting up Docker's GPG key...");
    SSH::execute_command(session, "sudo install -m 0755 -d /etc/apt/keyrings")?;
    SSH::execute_command(
      session,
      "sudo curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc",
    )?;
    SSH::execute_command(session, "sudo chmod a+r /etc/apt/keyrings/docker.asc")?;

    // Step 4: Add Docker repository
    println!("Adding Docker repository...");
    let add_repo_cmd = r#"echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/ubuntu $(. /etc/os-release && echo "${UBUNTU_CODENAME:-$VERSION_CODENAME}") stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null"#;
    let (exit_code, output) = SSH::execute_command(session, add_repo_cmd)?;
    if exit_code != 0 {
      return Err(anyhow::anyhow!(
        "Failed to add Docker repository: {}",
        output
      ));
    }

    // Step 5: Update package index again
    println!("Updating package index...");
    let (exit_code, output) = SSH::execute_command(session, "sudo apt-get update")?;
    if exit_code != 0 {
      return Err(anyhow::anyhow!(
        "Failed to update package index: {}",
        output
      ));
    }

    // Step 6: Install Docker
    println!("Installing Docker packages...");
    let install_cmd = "sudo apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin";
    let (exit_code, output) = SSH::execute_command(session, install_cmd)?;
    if exit_code != 0 {
      return Err(anyhow::anyhow!("Failed to install Docker: {}", output));
    }

    // Step 7: Add user to docker group
    println!("Adding user to docker group...");
    SSH::execute_command(session, "sudo groupadd docker")?; // This might fail if group already exists, but that's okay
    let add_user_cmd = format!("sudo usermod -aG docker {}", username);
    let (exit_code, output) = SSH::execute_command(session, &add_user_cmd)?;
    if exit_code != 0 {
      return Err(anyhow::anyhow!(
        "Failed to add user to docker group: {}",
        output
      ));
    }

    // Step 8: Verify docker installation
    println!("Verifying Docker installation...");
    let (exit_code, docker_version) = SSH::execute_command(session, "docker --version")?;
    if exit_code != 0 || docker_version.is_empty() {
      return Err(anyhow::anyhow!("Docker installation verification failed"));
    }

    println!("🐳 Docker version: {}", docker_version.trim());
    Ok(())
  }

  pub fn save_to_cluster(&self, session: &Session) -> anyhow::Result<()> {
    let cluster_init_json = serde_json::to_string_pretty(self)
      .context("Failed to serialize ClusterInit to JSON")?
      .replace("'", "'\\''");
    let write_cmd = format!(
      "mkdir -p ~/.dosei/doseid && cat > ~/.dosei/doseid/cluster-init.json << 'EOF'\n{}\nEOF",
      cluster_init_json
    );
    SSH::execute_command(session, &write_cmd)?;
    Ok(())
  }
}
