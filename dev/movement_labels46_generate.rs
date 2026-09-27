#[path = "../src/action_records.rs"]
mod action_records;
use action_records::MovementButtonGroup;
fn main() {
    for (name, label) in [("even", "Move"), ("odd", "Mover"), ("empty", "")] {
        let group =
            MovementButtonGroup::from_original_template_pairs(&[(1, 2), (2, 3)], label, (10, 5))
                .unwrap();
        std::fs::write(
            format!("/tmp/movement46_authored_label_{name}_group.bin"),
            group.encode(),
        )
        .unwrap();
        let mut original = MovementButtonGroup::parse(include_bytes!(
            "../tests/fixtures/movement46_cube_unlabeled_group.bin"
        ))
        .unwrap();
        original.set_target_pairs(&[(1, 2)]).unwrap();
        original.set_screen_anchor((10, 5));
        original.set_label(label).unwrap();
        std::fs::write(
            format!("/tmp/movement46_imported_label_{name}_group.bin"),
            original.encode(),
        )
        .unwrap();
    }
}
