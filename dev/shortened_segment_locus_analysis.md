# Shortened segment locus drivers

## Native record

`个人专栏/钮炳坤作品/沙漏之轨道移动版钮炳坤老师.gsp`
contains a code-35 locus at object 51. Its fixed roles are target 49, path 3,
and driver 16. Path 3 is segment `(1, 2)`, while driver 16 is constrained to
segment 15 `(14, 2)`. Point 14 is the fixed `0.01` dilation of point 2 about
point 1, so segment 15 is a shortened, collinear copy of path 3 which omits the
singular first endpoint.

The native locus domain remains the ordinary normalized segment interval
`0..1`. Sampling must keep driver 16 on its live host 15; replacing that host
with path 3 would reintroduce the endpoint which the construction deliberately
avoids. The saved path remains part of the locus dependency context.

The decoder therefore accepts a mismatched segment host only when it can prove
this exact relationship: the two segments share an endpoint and the other live
endpoint is a fixed scalar dilation, strictly between zero and one, of the
opposite source endpoint about the source segment's other endpoint. Transformed,
iterated, and unrelated constrained drivers remain opaque. This excludes the
known expensive general-driver loci such as `正态分布.gsp`.

## Native and corpus evidence

Sketchpad 5 under Xvfb/Wine displays the same blue rail and magenta sandglass
trace as the port. The disposable native copy opens with the expected title and
saves byte-for-byte with SHA-256
`71369f4f5895f4038c42bf9693cd0867be08c65d30aafa157f919b662f975d06`.

The compiled corpus scan restores the code-35 root plus nine dependent drawing
slots, including the nested locus, intersection, and literal locus point. The
unsupported total falls **853→843**, complete documents/drawings rise
**1,495/1,536→1,496/1,537**, and unsupported documents fall **104→103**. All
**1,599** originals remain byte-exact with zero failures; embedded-tool coverage
is unchanged. Full scan: `/tmp/shortened_segment_locus_coverage.log`.

No new test case was added.
