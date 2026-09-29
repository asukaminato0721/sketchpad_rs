# Duplicate seed columns in mapped iterations

The post-circle-locus frontier contained two ready code-89 mappings in
`课时7笔记.gsp` whose Native packets each repeat the same seed twice. The
corresponding branch rows also repeat the same image in both columns:

- object 765 stores seeds `[725, 725]` and branch `[762, 762]`;
- object 783 stores seeds `[725, 725]` and branch `[777, 777]`.

These columns do not describe two independently substituted points. They are
duplicate encodings of one substitution, just as repeated carried references
describe one construction DAG rather than repeated output. Import now keeps the
first occurrence of a seed only when every branch row has the same image in the
duplicate column. A repeated seed with different branch images remains rejected,
so the decoder does not collapse a potentially meaningful multi-image mapping.
Original reference records remain authoritative for unchanged saves.

The two canonical mappings drive four segment bindings. Object 765 iterates the
source point horizontally and binds segments 761 and 764; object 783 iterates it
vertically and binds segments 774 and 776. The compiled corpus scan restores
those two drawing code-89 definitions and four drawing code-77 bindings.
Unsupported slots fall 1,766→1,760 (1,162 drawing and 598 prototype), all-kind
complete documents rise 1,405→1,406, drawing-complete documents rise
1,441→1,442, unsupported documents fall 194→193, and partial views stay at 825.
All 1,599 originals remain byte-exact. The ready code-89 frontier falls from 17
roots/261 reachable unsupported slots to 15 roots/255 reachable slots.

Sketchpad 5 under Xvfb/Wine displays the source page's translated segment
construction and saves all 387,588 bytes unchanged. The SHA-256 before and after
Native Ctrl+S is
`09ea3df65cc6bc90678c4121d6f22c1e8cd53aa1373b4b38f3b792b7023b20a5`.
The capture is `/tmp/code89_duplicate_seed_native_before_save.png`; the port
decodes roots 765/783 and bindings 766/767/784/785. No standalone test case was
added.
