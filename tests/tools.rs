use rainy_cli::{tools::{execute_tool, ToolCall}, utils::diff::FileModification};
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
    let mut mods = Vec::new();
    let write_result = execute_tool(write_call, &mut mods).await.unwrap();
    assert!(write_result.success);
    assert!(test_file_path.exists());

    let read_call = ToolCall::ReadFile {
        path: test_file_path.to_str().unwrap().to_string(),
    };
    let read_result = execute_tool(read_call, &mut mods).await.unwrap();
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
    let list_result = execute_tool(list_call, &mut mods).await.unwrap();
    assert!(list_result.success);
    assert!(list_result.output.contains("[F] test.txt"));
    assert!(list_result.output.contains("[F] another.txt"));
    assert!(list_result.output.contains("[D] subdir"));


    // 3. Test Delete
    let delete_call = ToolCall::DeleteFile {
        path: test_file_path.to_str().unwrap().to_string(),
    };
    let delete_result = execute_tool(delete_call, &mut mods).await.unwrap();
    assert!(delete_result.success);
    assert!(!test_file_path.exists());

    // The tempdir will be automatically cleaned up when `dir` goes out of scope.
}

#[tokio::test]
async fn test_read_nonexistent_file() {
    let read_call = ToolCall::ReadFile {
        path: "nonexistent_file.txt".to_string(),
    };
    let mut mods = Vec::new();
    let read_result = execute_tool(read_call, &mut mods).await.unwrap();
    assert!(!read_result.success);
    assert!(read_result.output.contains("Failed to read file"));
}

#[tokio::test]
async fn test_grep_tool() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let file_path = root.join("grep_test.txt");
    let content = "Hello, world!\nThis is a test.\nAnother line with world.\n";
    fs::write(&file_path, content).unwrap();

    let grep_call = ToolCall::Grep {
        pattern: "world".to_string(),
        path: Some(file_path.to_str().unwrap().to_string()),
    };
    let mut mods = Vec::new();
    let grep_result = execute_tool(grep_call, &mut mods).await.unwrap();
    assert!(grep_result.success);
    assert!(grep_result.output.contains("1:Hello, world!"));
    assert!(grep_result.output.contains("3:Another line with world."));
}

#[tokio::test]
async fn test_patch_tool() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let file_path = root.join("patch_test.txt");
    let original_content = "Hello, world!\nThis is a test.\n";
    fs::write(&file_path, original_content).unwrap();

    let patch_instructions = "--- a/patch_test.txt
+++ b/patch_test.txt
@@ -1,2 +1,2 @@
-Hello, world!
-This is a test.
+Hello, patched world!
+This is a patched test.
";

    let patch_call = ToolCall::PatchFile {
        path: file_path.to_str().unwrap().to_string(),
        instructions: patch_instructions.to_string(),
    };
    let mut mods = Vec::new();
    let patch_result = execute_tool(patch_call, &mut mods).await.unwrap();
    assert!(patch_result.success);

    let patched_content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(patched_content, "Hello, patched world!\nThis is a patched test.\n");
}
