use crate::runtime::shell::ShellExecutor;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn run_command(
    command: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let (timeout_secs, sandbox_enabled, sandbox_image, working_dir) = {
        let config = state.config.read().await;
        (
            config.command_timeout_s,
            config.sandbox_enabled,
            config.sandbox_image.clone(),
            config.working_directory.clone(),
        )
    };

    let executor = ShellExecutor::new(timeout_secs, sandbox_enabled, sandbox_image, working_dir);

    let output = executor.run(&command).await.map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "stdout": output.stdout,
        "stderr": output.stderr,
        "exitCode": output.exit_code,
        "timedOut": output.timed_out,
    }))
}

#[tauri::command]
pub async fn cancel_command() -> Result<(), String> {
    // Note: For true cancellation we would need to track the process handle
    // For now this is a stub that returns success
    Ok(())
}
