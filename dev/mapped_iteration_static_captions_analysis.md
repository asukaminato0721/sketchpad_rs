# Static captions in mapped construction lists

`分解质因数.gsp` object 48 is a five-seed code-89 mapping whose Native
construction list includes two unbound referenced captions. Object 4 contains
the stable name of a parameter (`能否被 … 整除`), while object 47 contains the
stable expression of a parameter calculation (`记录被 … 整除的次数`). Neither
caption is an iteration image or introduces a substituted value; both are
presentation prerequisites retained by Native's carried list.

Unbound `ReferencedText` constructions are now admitted when every reference
role is `Name`, `Value`, or `Expression`. Name and expression rows resolve from
stable object metadata, while any value row continues through the existing
mapped live/frozen resolver. The stricter output rule is unchanged:
`PointReferencedText` bindings still require a live `Value`, and `Definition`
and `SignedValue` roles remain outside the proved mapped scope.

Admitting the root restores its dependent binding, alias, transform, label,
ratio, calculation, nested mapping, and final bindings: 13 drawing slots in
one chain. The port's final mapped numeric labels are 3, 11, and 61, matching
Sketchpad's displayed `2013 = 3 × 11 × 61`. Sketchpad 5 under Xvfb/Wine saves
the 11,082-byte control unchanged with SHA-256
`eefc28062d2babe448c596e3a621b05da37b970fee9f3e67e03735e9ed7ba836`.
Evidence is `/tmp/code89_expression_native_wow64_before_save.png` and
`/tmp/code89_expression_port_narrow.svg`.

Across the full 1,599-document corpus, unsupported slots fall 1,632→1,619
(1,021 drawing and 598 prototype), all-kind complete documents rise
1,410→1,411, drawing-complete documents rise 1,446→1,447, unsupported
documents fall 189→188, and partial views fall 824→823. Every original remains
byte-exact. The ready code-89 frontier falls from 9 roots/127 reachable
unsupported slots to 8 roots/114 reachable slots. No standalone test case was
added.
