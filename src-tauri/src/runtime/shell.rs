use crate::error::OpenCLIError;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

pub struct ShellExecutor {
    timeout_secs: u64,
    sandbox_enabled: bool,
    sandbox_image: Option<String>,
    working_dir: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub timed_out: bool,
}

impl ShellExecutor {
    pub fn new(
        timeout_secs: u64,
        sandbox_enabled: bool,
        sandbox_image: Option<String>,
        working_dir: Option<String>,
    ) -> Self {
        Self {
            timeout_secs,
            sandbox_enabled,
            sandbox_image,
            working_dir,
        }
    }

    pub async fn run(&self, command: &str) -> Result<CommandOutput, OpenCLIError> {
        let timeout_duration = Duration::from_secs(self.timeout_secs);

        let mut cmd = if self.sandbox_enabled {
            self.build_sandbox_command(command)
        } else {
            self.build_native_command(command)
        };

        if let Some(dir) = &self.working_dir {
            cmd.current_dir(dir);
        }

        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let result = timeout(timeout_duration, async {
            let child = cmd
                .spawn()
                .map_err(|e| OpenCLIError::Shell(e.to_string()))?;
            let output = child
                .wait_with_output()
                .await
                .map_err(|e| OpenCLIError::Shell(e.to_string()))?;
            Ok::<_, OpenCLIError>(output)
        })
        .await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code().unwrap_or(-1);
                Ok(CommandOutput {
                    stdout,
                    stderr,
                    exit_code,
                    timed_out: false,
                })
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err(OpenCLIError::CommandTimeout),
        }
    }

    fn build_native_command(&self, command: &str) -> Command {
        if cfg!(target_os = "windows") {
            let mut cmd = Command::new("cmd");
            cmd.args(["/C", command]);
            cmd
        } else {
            let mut cmd = Command::new("/bin/sh");
            cmd.args(["-c", command]);
            cmd
        }
    }

    fn build_sandbox_command(&self, command: &str) -> Command {
        let image = self.sandbox_image.as_deref().unwrap_or("alpine:latest");
        let mut cmd = Command::new("docker");
        cmd.args([
            "run",
            "--rm",
            "--cpus",
            "1",
            "--memory",
            "512m",
            "--network",
            "none",
            image,
            "/bin/sh",
            "-c",
            command,
        ]);
        cmd
    }
}
