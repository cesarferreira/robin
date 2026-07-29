use robin::makefile::{load_makefile_scripts, parse_makefile};
use robin::scripts::list_scripts;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn load_makefile_scripts_reads_targets_from_file() {
    let dir = tempdir().unwrap();
    std::fs::write(
        dir.path().join("Makefile"),
        "# Build it\nbuild:\n\tcargo build\n\ntest:\n\tcargo test\n",
    )
    .unwrap();

    let scripts = load_makefile_scripts(&dir.path().join("Makefile")).unwrap();
    assert!(scripts.contains_key("build"));
    assert!(scripts.contains_key("test"));
    assert_eq!(scripts["build"]["desc"].as_str().unwrap(), "Build it");
}

#[test]
fn list_scripts_prints_makefile_targets() {
    let content = "build:\n\techo hi\n";
    let scripts = parse_makefile(content, Path::new(".")).unwrap();
    let result = list_scripts(&scripts);
    assert!(result.is_ok());
}
