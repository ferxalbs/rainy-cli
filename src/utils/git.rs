use anyhow::Result;

pub fn get_git_changes(git_ref: &str) -> Result<Vec<(String, String)>> {
    let repo = git2::Repository::open(".")?;

    // Get the tree for the reference
    let obj = repo.revparse_single(git_ref)?;
    let tree = obj.peel_to_tree()?;

    // Get the current index (working directory + staging area)
    let index = repo.index()?;

    // Get diff between the tree and index
    let diff = repo.diff_tree_to_index(Some(&tree), Some(&index), None)?;

    let mut changed_files = Vec::new();

    diff.foreach(&mut |delta, _| {
        if let Some(path) = delta.new_file().path() {
            if let Some(ext) = path.extension() {
                // Only include common code files
                if matches!(ext.to_str(), Some("rs") | Some("js") | Some("ts") | Some("py") | Some("java") | Some("cpp") | Some("c") | Some("h")) {
                    if let Ok(content) = std::fs::read_to_string(path) {
                        changed_files.push((path.display().to_string(), content));
                    }
                }
            }
        }
        true
    }, None, None, None)?;

    // Also check for untracked files
    let mut status_options = git2::StatusOptions::new();
    status_options.include_untracked(true);

    let statuses = repo.statuses(Some(&mut status_options))?;

    for entry in statuses.iter() {
        if entry.status().contains(git2::Status::WT_NEW) {
            if let Some(path) = entry.path() {
                let path_buf = std::path::PathBuf::from(path);
                if let Some(ext) = path_buf.extension() {
                    if matches!(ext.to_str(), Some("rs") | Some("js") | Some("ts") | Some("py") | Some("java") | Some("cpp") | Some("c") | Some("h")) {
                        if let Ok(content) = std::fs::read_to_string(&path_buf) {
                            changed_files.push((path.to_string(), content));
                        }
                    }
                }
            }
        }
    }

    Ok(changed_files)
}

pub fn get_git_status_summary() -> Result<String> {
    let repo = git2::Repository::open(".")?;
    let mut summary = String::new();
    
    if let Ok(statuses) = repo.statuses(None) {
        let mut added = 0;
        let mut modified = 0;
        let mut deleted = 0;
        let mut untracked = 0;
        
        for entry in statuses.iter() {
            let status = entry.status();
            if status.contains(git2::Status::INDEX_NEW) {
                added += 1;
            } else if status.contains(git2::Status::INDEX_MODIFIED) || status.contains(git2::Status::WT_MODIFIED) {
                modified += 1;
            } else if status.contains(git2::Status::INDEX_DELETED) || status.contains(git2::Status::WT_DELETED) {
                deleted += 1;
            } else if status.contains(git2::Status::WT_NEW) {
                untracked += 1;
            }
        }
        
        summary.push_str(&format!("Git Status: {} added, {} modified, {} deleted, {} untracked", 
                                  added, modified, deleted, untracked));
    }
    
    Ok(summary)
}
