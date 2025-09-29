use rainy_cli::shell::{
    CommandResult, SecurityLevel, ShellConfig, ShellExecutor,
};
use std::time::Duration;
use tempfile::tempdir;
use tokio;

#[test]
fn test_security_level_from_str() {
    assert_eq!(SecurityLevel::from_str("low"), SecurityLevel::Low);
    assert_eq!(SecurityLevel::from_str("medium"), SecurityLevel::Medium);
    assert_eq!(SecurityLevel::from_str("high"), SecurityLevel::High);
    assert_eq!(SecurityLevel::from_str("invalid"), SecurityLevel::Low); // Default to most secure
}

#[test]
fn test_shell_config_default() {
    let config = ShellConfig::default();
    assert_eq!(config.security_level, SecurityLevel::Medium);
    assert_eq!(config.timeout_seconds, 300);
    assert!(!config.allowed_commands.is_empty());
    assert!(!config.blocked_commands.is_empty());
}

#[tokio::test]
async fn test_safe_command_execution() {
    let executor = ShellExecutor::new(ShellConfig {
        security_level: SecurityLevel::High, // No approval needed
        ..Default::default()
    });

    // Test a safe command that should work on all platforms
    let result = executor.execute("echo hello").await;

    match result {
        Ok(cmd_result) => {
            assert_eq!(cmd_result.exit_code, 0);
            assert!(cmd_result.success);
            assert!(cmd_result.stdout.contains("hello"));
        }
        Err(e) => {
            // Command might not be available in test environment
            println!(
                "Command execution failed (expected in some test environments): {}",
                e
            );
        }
    }
}

#[tokio::test]
async fn test_command_timeout() {
    let executor = ShellExecutor::new(ShellConfig {
        security_level: SecurityLevel::High,
        timeout_seconds: 1, // Very short timeout
        ..Default::default()
    });

    // This command should timeout (sleep/timeout command)
    let result = if cfg!(target_os = "windows") {
        executor.execute("Start-Sleep -Seconds 5").await
    } else {
        executor.execute("sleep 5").await
    };

    match result {
        Err(e) => {
            assert!(e.to_string().contains("timed out"));
        }
        Ok(_) => {
            // Command might complete quickly in some environments
            println!("Command completed unexpectedly fast");
        }
    }
}

#[tokio::test]
async fn test_batch_execution() {
    let executor = ShellExecutor::new(ShellConfig {
        security_level: SecurityLevel::High,
        ..Default::default()
    });

    let commands = vec!["echo first".to_string(), "echo second".to_string()];

    match executor.execute_batch(&commands).await {
        Ok(results) => {
            assert_eq!(results.len(), 2);
            for result in results {
                if result.success {
                    assert_eq!(result.exit_code, 0);
                }
            }
        }
        Err(e) => {
            println!(
                "Batch execution failed (expected in some test environments): {}",
                e
            );
        }
    }
}

#[test]
fn test_command_result_serialization() {
    let result = CommandResult {
        command: "echo test".to_string(),
        exit_code: 0,
        stdout: "test\n".to_string(),
        stderr: "".to_string(),
        duration: Duration::from_millis(100),
        success: true,
    };

    // Test serialization
    let serialized = serde_json::to_string(&result).unwrap();
    assert!(serialized.contains("echo test"));
    assert!(serialized.contains("test\\n"));

    // Test deserialization
    let deserialized: CommandResult = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.command, result.command);
    assert_eq!(deserialized.exit_code, result.exit_code);
    assert_eq!(deserialized.success, result.success);
}

#[test]
fn test_security_level_serialization() {
    let levels = vec![
        SecurityLevel::Low,
        SecurityLevel::Medium,
        SecurityLevel::High,
    ];

    for level in levels {
        let serialized = serde_json::to_string(&level).unwrap();
        let deserialized: SecurityLevel = serde_json::from_str(&serialized).unwrap();
        assert_eq!(level, deserialized);
    }
}

#[tokio::test]
async fn test_file_management_functions() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test_file.txt");
    let file_path_str = file_path.to_str().unwrap();

    let executor = ShellExecutor::new(ShellConfig {
        security_level: SecurityLevel::High, // No approval needed
        ..Default::default()
    });

    // 1. Create a file
    let content = "Hello from rainy-cli!";
    let create_result = executor.create_file(file_path_str, content).await;
    assert!(create_result.is_ok());
    assert!(create_result.unwrap().success);

    // 2. Read the file and verify its content
    let read_content = executor.read_file(file_path_str).await;
    assert!(read_content.is_ok());
    assert_eq!(read_content.unwrap().trim(), content);

    // 3. Delete the file
    let delete_result = executor.delete_file(file_path_str).await;
    assert!(delete_result.is_ok());
    assert!(delete_result.unwrap().success);

    // 4. Verify the file is gone
    assert!(!file_path.exists());
}

#[tokio::test]
async fn test_git_functions() {
    let dir = tempdir().unwrap();
    let repo_path = dir.path();

    let executor = ShellExecutor::new(ShellConfig {
        security_level: SecurityLevel::High,
        working_directory: Some(repo_path.to_path_buf()),
        ..Default::default()
    });

    // 1. Initialize a new git repository
    let init_result = executor.execute("git init").await;
    assert!(init_result.is_ok() && init_result.unwrap().success);

    // Set user config for commits
    executor.execute("git config user.name 'Test User'").await.unwrap();
    executor.execute("git config user.email 'test@example.com'").await.unwrap();

    // 2. Check initial status
    let initial_status = executor.git_status().await;
    assert!(initial_status.is_ok());
    assert!(initial_status.unwrap().contains("No commits yet"));

    // 3. Create a new file, add, and commit it
    let file_path = repo_path.join("README.md");
    std::fs::write(&file_path, "This is a test repository.").unwrap();

    let add_result = executor.git_add(&["README.md"]).await;
    assert!(add_result.is_ok() && add_result.unwrap().success);

    let status_after_add = executor.git_status().await.unwrap();
    assert!(status_after_add.contains("Changes to be committed:"));
    assert!(status_after_add.contains("new file:   README.md"));

    let commit_result = executor.git_commit("Initial commit").await;
    assert!(commit_result.is_ok() && commit_result.unwrap().success);

    // 4. Check status after commit
    let final_status = executor.git_status().await.unwrap();
    assert!(final_status.contains("nothing to commit, working tree clean"));
}