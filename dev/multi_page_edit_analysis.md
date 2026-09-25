# Multi-page in-place edits

The native fixture `tests/fixtures/two_page_two_points_native.gsp` has one free point on each page. Dragging the page 2 point in Sketchpad 5 from screen `(280,200)` to `(320,240)` produced `two_page_point_dragged_native.gsp`, moving its raw coordinates from `(221,131)` to `(261,171)`. The native file changed the point's `0x0899` payload and cleared three opaque bytes in a page `0x0386` record.

`tests/replay_multi_page_point_move.txt` generated the same moved point through Rust. Its output differs from the native dragged file only at those three opaque bytes. Native Sketchpad displayed the point in its new position and saved the Rust file byte-identically. The same replay moves a page 1 point and inserts a Page Link before page 2; that combined file also opened and saved byte-identically in native Sketchpad.

The multi-page writer patches existing free-point coordinates, visibility, and Hide/Show action state in place while preserving unsupported records. It validates the reparsed objects before replacing the file. Other geometric changes remain guarded.
