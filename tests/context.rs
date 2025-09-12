use rainy_cli::utils::agents_md::load_hierarchical_agents_md;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_hierarchical_agents_md() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create a AGENTS.md in the root
    let root_agents_md_path = root.join("AGENTS.md");
    let root_content = "This is the root AGENTS.md";
    fs::write(root_agents_md_path, root_content).unwrap();

    // Create a subdirectory and a AGENTS.md in it
    let sub_dir = root.join("subdir");
    fs::create_dir(&sub_dir).unwrap();
    let sub_agents_md_path = sub_dir.join("AGENTS.md");
    let sub_content = "This is the sub AGENTS.md";
    fs::write(sub_agents_md_path, sub_content).unwrap();

    // Change current directory to the subdirectory
    std::env::set_current_dir(&sub_dir).unwrap();

    let combined_content = load_hierarchical_agents_md().unwrap();

    // Check that both contents are present and the sub content is first
    assert!(combined_content.contains(root_content));
    assert!(combined_content.contains(sub_content));
    assert!(combined_content.find(sub_content).unwrap() < combined_content.find(root_content).unwrap());
}
