# Final drawing referenced captions

Audited 2026-09-30 against the complete 1,599-file corpus and Native Sketchpad
5 under Xvfb/Wine.

## Scope

Four drawing code-73 records remained. Three records in `课时9笔记.gsp` use
ordinary Value-role references to an independent picture or a sampled picture.
Their rich text is already within the supported grammar and carries Native's
localized generated names (`图片 [1]` and `图片 [3]`) as explicit caches. The
sources were rejected only because picture kinds were absent from the Value
source contract.

The fourth record, in `时钟 (2).gsp`, uses the exact legacy 16-byte
class-0/subtype-59 caption header and otherwise has the established
refs/rich/font/terminator layout. Its six parents alternate live scalar
calculations with literal `时`, `分`, and `秒` text objects. The rich caption is
GBK and resolves to `4时19分41秒` for the saved state.

## Admission and encoding boundary

Picture Value sources are admitted only after normal earlier-object, same-page,
decoded-parent validation. Evaluation uses the native cached generated name;
bare API-authored references receive a page-local `Picture [n]` fallback.
Other unsupported source kinds still cannot enter through cached display text.

The legacy caption admission is limited to its complete observed 16-byte
header. GBK rich literals remain length-checked by the shared parser. Legacy
free-text objects use document-cohort detection: at least one exact
short-header text object must contain markup that is unambiguously not UTF-8
before other exact short-header strings are forced through code page 936. This
disambiguates `CA B1`, which is `时` in GBK but also happens to encode `ʱ` in
UTF-8. The original rich bytes and encoding mode stay together in the text
template, so unrelated edits do not rewrite the legacy source.

## Evidence

The port SVGs contain the three complete instructional captions and the clock
caption `4时19分41秒`; neither contains an undefined reference. Both original
files open in Native Sketchpad 5 and Ctrl-S byte-for-byte under Xvfb/Wine:

- `课时9笔记.gsp`:
  `cd793b1118c29f81e811fcadfc83022cc9d40b9bd53aad92ca7d23669e19c0ac`
- `时钟 (2).gsp`:
  `c95adfc4606abb897b28b4cb2f6393f7318cd59005cbf0b1cb8a0ff4f7c4993d`

The complete compiled scan restores exactly four drawing slots. Unsupported
falls 587 to 583; complete documents rise 1,553 to 1,555 and drawing-complete
documents rise 1,592 to 1,594. All 1,599 files remain whole-file exact with
zero import failures. Embedded-tool coverage is unchanged at 4,779 of 5,356.
No standalone test case was added. The scan is
`/tmp/referenced_caption_coverage_full.log`; native captures are
`/tmp/referenced_lesson_native_before_save.png` and
`/tmp/referenced_clock_native_before_save.png`.
