# Mapped name-and-value captions

The remaining ready code-89 frontier included one mapping on page 17 of
`基础迭代26例（hjx4882）.gsp`. Its carried object 1256 is a referenced caption
with two roles for the same calculation: a static `Name` (`余数`) followed by a
live `Value`. Object 1257 anchors that caption to the mapped point, root 1258
iterates it, and code-77 bindings 1259/1260 expose the coordinate and remainder
labels.

Mapped referenced text now accepts a mixture of `Name` and `Value` roles when
at least one live `Value` is present. Names resolve from stable object metadata;
values continue through the existing substituted live/frozen evaluator.
Name-only captions are not admitted as mapped values, and `Definition`,
`Expression`, and `SignedValue` retain their separate unsupported scope
contracts. The expression-bearing caption in `分解质因数.gsp` therefore remains
outside this change.

On the selected Native page, Sketchpad displays remainder labels 0 through 5
across each row. The port SVG produces the same repeating sequence
`余数=0` through `余数=5`, alongside the existing coordinate labels. A page-17
copy opened and saved under Sketchpad 5 with Xvfb/Wine retains all 314,698 bytes
and SHA-256
`39ba58f3b951ed6b241a1b173bc9bcfba23fb309e117be8a3a975ce6d76085dd`.
Captures are `/tmp/code89_name_value_native_large_after_action.png` and
`/tmp/sketchpad_code89_name_value_page17.svg`.

The compiled corpus restores one drawing code-89 definition and two drawing
code-77 bindings. Unsupported slots fall 1,760→1,757 (1,159 drawing and 598
prototype); document completeness and partial-view counts remain 1,406
all-kind, 1,442 drawing, 193 unsupported, and 825 partial because the same file
still contains an independent code-89 mapping. All 1,599 originals remain
byte-exact. The ready code-89 frontier falls from 15 roots/255 reachable
unsupported slots to 14 roots/252 reachable slots. No standalone test case was
added.
