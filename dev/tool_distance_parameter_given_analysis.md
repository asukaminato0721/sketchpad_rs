# Distance-Parameter custom-tool given

`杨辉三角1（李章博）.gsp` contains the only class-11/subtype-80 custom-tool
given in the 1,599-document corpus. Its five-object prototype declares input
`[1]`, followed by two code-48 calculations and two movement buttons.

Native Sketchpad 5 Script View names the prerequisite `距离参数 行距` and says
to click a distance Parameter to match it. This establishes subtype 80 as a
Parameter-specific category, not the general distance-value category used by
subtype 48. The port therefore represents it as `DistanceParameter`, assigns
it Length units, and requires a live `Kind::Parameter` whose unit is Length in
both engine invocation and desktop hit-testing.

The placeholder retains the strict six-record parameter-given family and is
validated without treating its cache bytes as an authored value or animation
domain. The two dependent calculations specialize to Length when the tool is
bound.

A focused engine invocation copied the source `行距` Parameter into a clean
document and materialized both calculations plus both movement buttons. The
2,716-byte result encoded and reparsed with five objects. Native Sketchpad
rendered `行距 = 0.74 cm` and both buttons, preserved the file byte-for-byte on
Ctrl+S with SHA-256
`c1981ec51c4fddd460fb765392e4920d2c1ee35d6dad9a81afbf77771822fe19`,
and changed the value to `0.75 cm` when the first button was clicked.

Corpus coverage rises from 5,065 to 5,068 decoded prototype slots out of
5,356; unsupported prototype slots fall from 291 to 288. Complete isolated
views rise from 48 to 49, and fully decoded documents from 1,571 to 1,572. All
1,599 originals remain byte-exact and all drawing objects remain decoded. The
full scan is `/tmp/subtype80_runtime_coverage.txt`.
