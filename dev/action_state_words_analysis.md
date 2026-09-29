# Movement and Scroll action state words

## Observed packets

Two otherwise ordinary code-62 action buttons store `1` in the final state
word of their 24-byte `0x0906` payload. The Movement button in
`勾股定理小题.gsp` has speed `Medium`, follows its live destination, and targets
one decoded `PointOnSegment`/translated-point pair. The Scroll button in
`基础迭代26例（hjx4882）.gsp` uses the upper-left placement and targets one
decoded transformed point. Their object headers, references, labels, auxiliary
records, and all other action words match the existing strict readers.

The readers now accept only state words `0` and `1`. Imported groups retain the
word when their editable fields are patched. Freshly authored and copied
buttons continue to use the Native template's zero state; no new document-model
state or runtime behavior is inferred from the retained word.

## Native and corpus evidence

Sketchpad 5 under Xvfb/Wine opens and saves both original state-1 documents
byte-for-byte. Their SHA-256 values are respectively
`6e88700d4461eb5e8f2663ebba77c4aff1271b9fbd28b2acbf60a45e78514c9c`
and
`ce52fadb3378082b737a546fb9073a0679b1bda43d67df2b287b03d3a9eac928`.

The compiled corpus scan restores the two action roots and one dependent
caption. Unsupported falls **817→814**, code-62 drawing objects fall
**31→29**, complete documents/drawings rise **1,499/1,540→1,501/1,542**, and
unsupported documents fall **100→98**. Partial views remain **802**, embedded
tool coverage is unchanged, and all **1,599** originals remain byte-exact with
zero failures. Full scan: `/tmp/action_state_coverage.log`.

No new test case was added.
