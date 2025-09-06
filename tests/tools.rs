use rainy_cli::tools::{execute_tool, ToolCall};
use std::fs;
use tempfile::tempdir;

#[tokio::test]
async fn test_file_operations() {
    // Create a temporary directory for the test
    let dir = tempdir().unwrap();
    let root = dir.path();

    // 1. Test Write and Read
    let test_file_path = root.join("test.txt");
    let test_content = "Hello, world!";

    let write_call = ToolCall::WriteFile {
        path: test_file_path.to_str().unwrap().to_string(),
        content: test_content.to_string(),
    };
    let write_result = execute_tool(write_call).await.unwrap();
    assert!(write_result.success);
    assert!(test_file_path.exists());

    let read_call = ToolCall::ReadFile {
        path: test_file_path.to_str().unwrap().to_string(),
    };
    let read_result = execute_tool(read_call).await.unwrap();
    assert!(read_result.success);
    assert_eq!(read_result.output, test_content);

    // 2. Test List
    let another_file = root.join("another.txt");
    fs::write(&another_file, "some data").unwrap();
    let sub_dir = root.join("subdir");
    fs::create_dir(&sub_dir).unwrap();


    let list_call = ToolCall::ListFiles {
        path: root.to_str().unwrap().to_string(),
    };
    let list_result = execute_tool(list_call).await.unwrap();
    assert!(list_result.success);
    assert!(list_result.output.contains("[F] test.txt"));
    assert!(list_result.output.contains("[F] another.txt"));
    assert!(list_result.output.contains("[D] subdir"));


    // 3. Test Delete
    let delete_call = ToolCall::DeleteFile {
        path: test_file_path.to_str().unwrap().to_string(),
    };
    let delete_result = execute_tool(delete_call).await.unwrap();
    assert!(delete_result.success);
    assert!(!test_file_path.exists());

    // The tempdir will be automatically cleaned up when `dir` goes out of scope.
}

#[tokio::test]
async fn test_read_nonexistent_file() {
    let read_call = ToolCall::ReadFile {
        path: "nonexistent_file.txt".to_string(),
    };
    let read_result = execute_tool(read_call).await.unwrap();
    assert!(!read_result.success);
    assert!(read_result.output.contains("Failed to read file"));
}
