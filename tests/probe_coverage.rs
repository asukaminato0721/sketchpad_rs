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
    assert!(report.contains("Unsupported drawing object codes"));
    assert!(report.contains("tests/fixtures/example_tools.gsp"));
    assert!(
        report.contains(
            "Embedded tool prototypes: 8 definitions in 1 documents; 177 slots, 141 decoded, 36 undecoded"
        )
    );
    assert!(report.contains("object slots: 524"));
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

#[test]
fn coverage_counts_known_and_unknown_tool_prototypes_in_separate_namespaces() {
    let output = Command::new(env!("CARGO_BIN_EXE_gsp_probe"))
        .args([
            "coverage",
            "tests/fixtures/point.gsp",
            "tests/fixtures/embedded_tool_point_scene_original_native_verified.gsp",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let report = String::from_utf8(output.stdout).unwrap();
    assert!(report.contains("1 with all object kinds decoded, 1 with unsupported objects"));
    assert!(report.contains("object slots: 46"));
    assert!(
        report.contains("Drawing objects: 2 slots; 2 documents with all drawing kinds decoded")
    );
    assert!(report.contains("44 slots, 36 decoded, 8 undecoded"));
    assert!(report.contains("Isolated tool views: 0 complete, 1 partial, 0 failed"));
    assert!(report.contains("Unsupported object slots including tool prototypes: 8"));
    assert!(report.contains("Unsupported tool prototype codes"));
}

#[test]
fn coverage_recognizes_complete_native_square_and_simple_prototypes() {
    let output = Command::new(env!("CARGO_BIN_EXE_gsp_probe"))
        .args([
            "coverage",
            "tests/fixtures/embedded_tool_square_registered_native_verified.gsp",
            "tests/fixtures/embedded_tool_simple_created_native_verified.gsp",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let report = String::from_utf8(output.stdout).unwrap();
    assert!(report.contains(
        "Documents: 2 total, 2 with all object kinds decoded, 0 with unsupported objects, 0 failed"
    ));
    assert!(report.contains("Byte-preserving round trips: 2/2; object slots: 22"));
    assert!(
        report.contains("Drawing objects: 5 slots; 2 documents with all drawing kinds decoded")
    );
    assert!(report.contains("17 slots, 17 decoded, 0 undecoded"));
    assert!(report.contains("Isolated tool views: 2 complete, 0 partial, 0 failed"));
    assert!(report.contains("Unsupported object slots including tool prototypes: 0"));
}

#[test]
fn coverage_reports_failed_tool_decode_without_failing_preserved_drawing() {
    let mut bytes =
        include_bytes!("fixtures/embedded_tool_simple_created_native_verified.gsp").to_vec();
    let declaration = sketchpad_rs::document::records(&bytes)
        .unwrap()
        .into_iter()
        .find(|record| record.kind == 0x04b4)
        .unwrap();
    bytes[declaration.data.start + 8..declaration.data.start + 12]
        .copy_from_slice(&2u32.to_le_bytes());
    let path = std::env::temp_dir().join(format!(
        "sketchpad_failed_tool_coverage_{}.gsp",
        std::process::id()
    ));
    fs::write(&path, bytes).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_gsp_probe"))
        .arg("coverage")
        .arg(&path)
        .output()
        .unwrap();
    fs::remove_file(path).unwrap();
    assert!(output.status.success());
    let report = String::from_utf8(output.stdout).unwrap();
    assert!(report.contains(
        "Documents: 1 total, 0 with all object kinds decoded, 1 with unsupported objects, 0 failed"
    ));
    assert!(report.contains("Byte-preserving round trips: 1/1; object slots: 8"));
    assert!(
        report.contains("Drawing objects: 4 slots; 1 documents with all drawing kinds decoded")
    );
    assert!(report.contains("4 slots, 0 decoded, 4 undecoded"));
    assert!(report.contains("Isolated tool views: 0 complete, 0 partial, 1 failed"));
    assert!(report.contains("Unsupported object slots including tool prototypes: 4"));
}

#[test]
fn coverage_keeps_partial_tool_metadata_separate_from_known_object_kinds() {
    let mut bytes =
        include_bytes!("fixtures/embedded_tool_simple_created_native_verified.gsp").to_vec();
    let entries = sketchpad_rs::document::records(&bytes).unwrap();
    let start = entries
        .iter()
        .position(|record| record.kind == 0x03ea)
        .unwrap();
    let first = entries[start..]
        .iter()
        .find(|record| record.kind == 0x07d0)
        .unwrap()
        .start;
    let unknown_metadata = [0u32.to_le_bytes(), 0xdeadu32.to_le_bytes()].concat();
    bytes.splice(first..first, unknown_metadata);
    let path = std::env::temp_dir().join(format!(
        "sketchpad_partial_tool_coverage_{}.gsp",
        std::process::id()
    ));
    fs::write(&path, bytes).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_gsp_probe"))
        .arg("coverage")
        .arg(&path)
        .output()
        .unwrap();
    fs::remove_file(path).unwrap();
    assert!(output.status.success());
    let report = String::from_utf8(output.stdout).unwrap();
    assert!(
        report.contains("1 with all object kinds decoded, 0 with unsupported objects, 0 failed")
    );
    assert!(report.contains("Byte-preserving round trips: 1/1"));
    assert!(report.contains("4 slots, 4 decoded, 0 undecoded"));
    assert!(report.contains("Isolated tool views: 0 complete, 1 partial, 0 failed"));
    assert!(report.contains("Unsupported object slots including tool prototypes: 0"));
}
