# Legacy and high-rate Animation buttons (code 62)

## Corpus evidence

After the ordinary Animation reader was complete, 12 action-code-2 groups had
supported targets but rejected settings. Ten use the modern 28-byte object
header. Eight of those are parameter reset buttons whose finite rates range
from `37,795.277` through `377,952.75` over correspondingly tiny domains. The
other two use settings that Native can execute but does not expose through its
authoring dialog.

The remaining two valid groups are in `时钟 (2).gsp`. They use a 16-byte code-62
header with the fixed class/subtype prefix `10, 23, 1`, ordinary valid Animation
settings, and labels stored in code page 936 rather than UTF-8:

- `b9 e9 c1 e3` — `归零`
- `bc c6 ca b1` — `计时`

The legacy header tail remains opaque and byte-preserved. GBK fallback is
limited to this 16-byte Animation-header cohort. An unchanged label is not
rewritten during unrelated edits, so a dirty save retains the original bytes.

## Admission boundary

Point Animation retains its finite maximum of `10,000`. Parameter Animation
accepts finite rates through `400,000`, covering the observed reset cohort
without removing the bound. Parameter domains and steps remain finite and
ordered, target types remain exact, and the record-layout checks are unchanged.

Native's authoring contract still requires a positive rate and disables Once
Only for bidirectional motion. The port applies those same restrictions to new
buttons. Import additionally admits the two observed executable forms:

- a point setting whose bidirectional mode makes its stored Once bit inert;
- a three-target packet whose zero-rate target remains stationary while the
  other two targets animate.

Both raw settings remain writable, so unrelated edits do not normalize their
bytes. The authoring restriction is documented by the
[Native Animation Properties reference](https://referencecenter.dynamicgeometry.com/gsp-reference-center/objects_btns_animateprops.htm).

## Verification

The real port editor opened the legacy clock under Xvfb and displayed both
decoded buttons. Dirty save/reopen checks preserved the two legacy groups and
the high-rate reset buttons, including their exact rates and parameter domains.
Clicking the first legacy button entered the existing Animation runtime.

The supplied Chinese Sketchpad 5 build was also run under Xvfb against the two
edge files. `位似(田野风).gsp` continuously moved its bidirectional point for at
least 12 seconds, proving that the stored Once bit is ignored. In
`螺旋迭代【3D】.gsp`, the three-target button was active on open: the positive-rate
targets animated while the zero-rate target stayed fixed, and clicking the
button stopped the group. Ctrl-S preserved the first source exactly; the second
source's runtime state was rewritten by Native, but its Animation settings were
unchanged.

Corpus coverage changed as follows:

- unsupported slots: **751 → 741**
- fully decoded documents: **1,513 → 1,514**
- documents with unsupported objects: **86 → 85**
- drawing-complete documents: **1,554 → 1,555**
- partial views: **800 → 800**
- byte-preserving originals: **1,599 / 1,599**

The later Sound-button and drawing-completion passes leave these two settings as
the final drawing gap. Admitting their observed runtime semantics changes the
current corpus totals as follows:

- unsupported slots: **579 → 577**, all in embedded-tool prototypes
- fully decoded documents: **1,557 → 1,559**
- drawing-complete documents: **1,597 → 1,599**
- documents with unsupported objects: **42 → 40**
- partial views: **787 → 785**
- byte-preserving originals: **1,599 / 1,599**

Embedded-tool coverage remains **4,779 / 5,356**. No standalone test case was
added.
