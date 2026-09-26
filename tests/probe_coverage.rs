use std::{fs, process::Command};

#[test]
fn coverage_distinguishes_partial_decoding_from_byte_preservation() {
    let output = Command::new(env!("CARGO_BIN_EXE_gsp_probe"))
        .args([
            "coverage",
            "tests/fixtures/point.gsp",
            "tests/fixtures/example_tools.gsp",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = String::from_utf8(output.stdout).unwrap();
    assert!(report.contains(
        "Documents: 2 total, 1 with all object kinds decoded, 1 with unsupported objects, 0 failed"
    ));
    assert!(report.contains("Byte-preserving round trips: 2/2"));
    assert!(report.contains("Unsupported object codes"));
    assert!(report.contains("tests/fixtures/example_tools.gsp"));
}

#[test]
fn coverage_checks_directories_and_reports_invalid_documents() {
    let directory = std::env::temp_dir().join(format!("sketchpad_coverage_{}", std::process::id()));
    fs::create_dir_all(directory.join("nested")).unwrap();
    fs::copy(
        "tests/fixtures/point.gsp",
        directory.join("nested/point.GSP"),
    )
    .unwrap();
    fs::write(directory.join("broken.gsp"), b"invalid document").unwrap();
    fs::write(directory.join("ignored.txt"), b"not a sketch").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_gsp_probe"))
        .arg("coverage")
        .arg(&directory)
        .output()
        .unwrap();
    fs::remove_dir_all(directory).unwrap();
    assert!(!output.status.success());
    let report = String::from_utf8(output.stdout).unwrap();
    assert!(report.contains(
        "Documents: 2 total, 1 with all object kinds decoded, 0 with unsupported objects, 1 failed"
    ));
    assert!(report.contains("Byte-preserving round trips: 1/2"));
    let errors = String::from_utf8(output.stderr).unwrap();
    assert!(errors.contains("broken.gsp"));
    assert!(errors.contains("1 document(s) failed"));
}

#[test]
fn coverage_separates_multi_page_edit_restrictions_from_unsupported_objects() {
    let output = Command::new(env!("CARGO_BIN_EXE_gsp_probe"))
        .args([
            "coverage",
            "tests/fixtures/two_page_text_port_saved_by_wine.gsp",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let report = String::from_utf8(output.stdout).unwrap();
    assert!(report.contains("1 with all object kinds decoded, 0 with unsupported objects"));
    assert!(report.contains("Partial views: 1/1"));
}
