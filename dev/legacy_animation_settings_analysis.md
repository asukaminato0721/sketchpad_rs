# Legacy and high-rate Animation buttons (code 62)

## Corpus evidence

After the ordinary Animation reader was complete, 12 action-code-2 groups had
supported targets but rejected settings. Ten use the modern 28-byte object
header. Eight of those are parameter reset buttons whose finite rates range
from `37,795.277` through `377,952.75` over correspondingly tiny domains. Two
others encode settings that remain contradictory or incomplete and are not
admitted.

The remaining two valid groups are in `时钟 (2).gsp`. They use a 16-byte code-62
header with the fixed class/subtype prefix `10, 23, 1`, ordinary valid Animation
settings, and labels stored in code page 936 rather than UTF-8:

- `b9 e9 c1 e3` — `归零`
- `bc c6 ca b1` — `计时`

The legacy header tail remains opaque and byte-preserved. GBK fallback is
limited to this 16-byte Animation-header cohort. An unchanged label is not
rewritten during unrelated edits, so a dirty save retains the original bytes.

## Admission boundary

Point Animation retains its existing finite positive maximum of `10,000`.
Parameter Animation accepts finite positive rates through `400,000`, covering
the observed reset cohort without removing the bound. All existing direction,
Once Only, parameter-domain, step, target-type, and exact record-layout checks
still apply.

Two packets deliberately remain unsupported:

- a point setting combines Once Only with bidirectional motion, which the
  native Animation Properties contract does not allow;
- a three-target packet contains a zero-speed settings block.

No semantics are invented for either packet. The native reference for the
Once Only restriction is
<https://referencecenter.dynamicgeometry.com/gsp-reference-center/objects_btns_animateprops.htm>.

## Verification

The real port editor opened the legacy clock under Xvfb and displayed both
decoded buttons. Dirty save/reopen checks preserved the two legacy groups and
the high-rate reset buttons, including their exact rates and parameter domains.
Clicking the first legacy button entered the existing Animation runtime.

Corpus coverage changed as follows:

- unsupported slots: **751 → 741**
- fully decoded documents: **1,513 → 1,514**
- documents with unsupported objects: **86 → 85**
- drawing-complete documents: **1,554 → 1,555**
- partial views: **800 → 800**
- byte-preserving originals: **1,599 / 1,599**

The remaining 19 drawing code-62 slots comprise 17 Sound buttons and the two
ambiguous Animation packets above. Embedded-tool coverage remains
**4,766 / 5,356**. No standalone test case was added.
