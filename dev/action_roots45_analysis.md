# Checkpoint 45 drawing action dependency-root audit

Read-only audit on the checkpoint 44 worktree. No action decoder, evaluator,
writer, or runtime changes and no new Native probe in this audit.

`dev/action_roots_audit45.rs` scans `Document.objects`, so embedded custom-tool
prototypes are excluded. It locates each unsupported code-62 group's actual
record bytes using `header_offset`, accounts for page-local references, runs
all five current strict action parsers, and reports missing/decoded parents.
Its output is `/tmp/action_roots_audit45.log`. `dev/action_roots45_htm.py`
produces `/tmp/action_roots_audit45_htm.log`, containing the original action
payloads and complete paired HTM contents for four selected source documents.
The Jinhua source has no paired HTM and is explicitly marked missing.

## Inventory

There are 5540 unsupported drawing action objects. 3064 have nonzero backward
references whose parent objects are all decoded. This is a structural upper
bound, not a claim that the parents support the requested action or that all
record variants have verified semantics.

| Raw action low word | Family | Unsupported | All parents decoded |
| --- | --- | ---: | ---: |
| 0 | Hide visibility variants | 1585 | 715 |
| 1 | Show visibility variants | 1934 | 853 |
| 2 | Animation | 197 | 182 |
| 3 | Movement | 1285 | 1131 |
| 4 | Scroll/focus | 97 | 97 |
| 7 | Presentation/sequence | 425 | 75 |
| 8 | Play function | 17 | 11 |

Ready movement counts by raw speed word are 10/164/70/887 for codes 0/1/2/3.
The supported native `MovementSpeed` mapping is Slow/Medium/Fast/Instant.
HTM has its own speed codes: for example the number-axis reset has raw speed1
but exported `MoveButton(...,3,...)`; they must not be conflated.

The most concrete next family is movement with a list of pairs, combined with
shared acceptance of an absent label record and an empty `0x232e` record.
1131 ready movement objects include only 74 one-pair Free-point sources and
31 multi-pair objects whose sources are exclusively Free points. Most require
existing point-on-path or numeric/graph-control setters. Their source kinds
are reported in the audit, so a Free-only extension would recover a small
minority of the family. There are 28 ready movement objects whose sources are
exclusively Parameters (14 one-pair, 14 two-pair).

## Exact original evidence

`个人专栏/况永胜作品/正方体的展开（3D效果）.gsp`, drawing object42:

- Records: `07d0:28,07d2:28,0906:24,07d5:34,07d8:2,07d7:0`.
- Raw references with the leading count removed: `[8,28,12,2,22,41]`.
- Action bytes: `06090000180000000c00000003000300000000000a000500`.
- Label: `左视图`.
- Paired HTM contains `{22} MoveButton(10,53,0,'左视图')(21,13,2,12,15,8)`.
  The HTM's object IDs and display coordinates differ from the original raw
  graph, but it explicitly proves a movement construction with three pairs.
  Objects64 and67 similarly contain six references and labels `主视图` and
  `俯视图`, with corresponding three-pair MoveButton definitions in HTM.
- Object62 has refs `[58,61]`, the same speed3 action bytes, and exactly
  `07d0:28,07d2:12,0906:24,07d8:2,07d7:0`: its `07d5` is absent.
  It is a linked action control with an empty label, not a truncated group.

`个人专栏/周维波作品/2010福建宁德(飞狐制作).gsp`, objects117/286/374:

- All omit `07d5`; raw reference counts are 8/12/10.
- Action bytes: `06090000180000000c000000030003000000000011008802`.
- Object117 refs `[40,36,77,87,64,116,51,115]`; records
  `07d0:28,07d2:36,0906:24,07d8:2,07d7:0`.
- Paired HTM explicitly exports empty-label MoveButtons with 8/12/10 refs:
  `{107} MoveButton(17,497,0,'')(105,41,106,54,77,67,26,30)`;
  `{276} MoveButton(17,497,0,'')(126,129,111,143,109,114,190,193,205,208,108,222)`;
  `{364} MoveButton(17,497,0,'')(339,343,340,354,303,304,281,314,282,325)`.
- Object376 has an absent `07d5`, refs `[30,118,287]`, and action
  `06090000180000000c000000000003000000000011008802`.
  The paired HTM `{365} HideButton(17,497,'')(20,108,277)` establishes a
  fixed Hide construction for at least this raw `(0,3)` variant. It does not
  establish the meanings of all seven option values.

`个人专栏/唐家军作品/课时6笔记.gsp`, drawing object367, page5:

- Global refs `[284,366]`, both decoded (GraphVerticalUnit and transformed point).
- Records `07d0:28,07d2:12,0906:24,07d5:40,07d8:2,232e:0,07d7:0`.
- Raw action `(3,3,0)`. Strict parser fails only at the extra empty `232e`.
- No adjacent paired HTM was found. This is wire-layout evidence, not enough
  alone to claim runtime equivalence. The inventory contains 240 ready
  movement records with this strict-parser failure class (also larger pair
  lists). This must be verified with a Native open/save or equivalent proven
  empty-metadata convention before generalizing the parser.

## Shared parser and runtime implications

Current action parsers store fixed arrays and require the exact six-record
layout (Animation requires seven). This rejects both optional label omission
and the empty `232e` suffix. A shared record representation should preserve
all original bytes and optional-record positions, while still rejecting
unknown nonempty data. Do not synthesize labels into imported groups just to
make the parser accept them.

The current MovementButton model accepts exactly two references and its
validation requires a single-page Free source. The ordinary action currently
moves a Free point; broad movement needs the existing constrained-point,
parameter, and graph-control editing semantics. A list representation avoids
an enum variant for every source kind and works with sequential/simultaneous
presentation actions. Backward refs and same-page roles must remain checked.

Visibility options are unresolved. The sibling `buttons.rs` treats every
`(0..=1,0..=7)` as one visibility toggle and discards option bits, so it is not
a reliable semantics oracle. In particular HTM explicitly distinguishes
HideButton from ToggleVisibilityButton. The existing default `(0 or1,7)`
changes its next action after a click; silently extending that behavior to
all options would be an unsupported guess.

## Native follow-up controls

1. Native-authored two/three-pair movement with Free and point-on-path sources,
   at Instant and one animated speed; compare changed source coordinates and
   path parameters, then move a destination with follow enabled.
2. Open/save original absent-label movement and empty-232e movement; preserve
   both record layouts on port import/export. Native action activation should
   also prove whether target positions use a shared initial state or earlier
   pair results when a source is another pair's destination.
3. One/two-pair numeric parameter movement and graph-axis unit movement, using
   paired HTM as construction evidence and Native click/save for setter roles.
4. Native options dialog controls for all Hide/Show option bits; compare
   payload bytes, button next action and label after one/two clicks. Include
   fixed Hide/Show with custom/empty labels and default toggle. Preserve raw
   options until their behavior has direct evidence.

These are planned controls, not completed Native proofs. Copy and custom-tool
support are outside this audit's requested scope.
