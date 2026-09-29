# Wider mapped-iteration branch tables

Four ready code-89 mappings use valid Native branch tables wider than the
port's former eight-row admission bound:

- `皮—希曲线2详解(inRm).gsp` object 93 has nine branches and depth 1;
- `密铺四边形(2)（hjx4882）.gsp` object 44 has nine branches and depth 2;
- `谢氏方形垫片换行系列迭代.gsp` object 45 has nine branches and depth 3;
- `基础迭代26例（hjx4882）.gsp` object 1337 has eleven branches and depth 2.

The packet schema already stores the branch count as a 32-bit value, and the
mapping model has no eight-branch representation limit. Decoder and validator
admission now use the same conservative 16-branch bound as the existing
16-seed bound. Runtime protection is unchanged: mapped evaluation refuses more
than 20,000 states in one level or 100,000 generated states overall. The four
observed controls generate at most 729 paths before coincident-state
deduplication.

The compiled corpus restores four drawing code-89 roots and 19 drawing code-77
bindings. Unsupported slots fall 1,757→1,734 (1,136 drawing and 598 prototype),
all-kind complete documents rise 1,406→1,409, drawing-complete documents rise
1,442→1,445, unsupported documents fall 193→190, and partial views fall
825→824. All 1,599 originals remain byte-exact. The ready code-89 frontier
falls from 14 roots/252 reachable unsupported slots to 10 roots/229 reachable
slots.

The nine-branch Peano control is a compact visual proof. Sketchpad 5 under
Xvfb/Wine and the port SVG display the same black/red recursive path and points
1 through 15. A page-4 copy saves all 19,590 bytes unchanged in Native with
SHA-256
`1d53ccf9fd379e87f850a414c0502b7299226efbafa527894b71932087d57081`.
Captures are `/tmp/code89_wide_peano_native_after_action.png` and
`/tmp/code89_wide_peano_page4.svg`. The other three port pages also export
within the existing resource caps. No standalone test case was added.
