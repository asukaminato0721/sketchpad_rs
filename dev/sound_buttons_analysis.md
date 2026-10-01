# Sound action buttons (code 62)

## Native structure and admission boundary

The corpus contains 17 Sound action groups. Every observed group targets one
function, while the Sketchpad 5 Sound Button documentation permits either one
function for mono playback or two functions for stereo playback. The strict
reader therefore accepts exactly one or two distinct, nonzero references, and
the document decoder additionally requires every reference to resolve to a
supported function.

The admitted group uses the ordinary 28-byte code-62 object header and the
standard action-record order. Its `0906` record is exactly 24 bytes and contains
a 12-byte payload block, action word `8`, six zero bytes, and the screen anchor.
The auxiliary record remains two bytes and the terminator remains empty. Other
code-62 layouts are not inferred from the Sound semantics.

Imported labels and opaque records remain byte-preserved. New buttons are built
from the established action template with a default `Play Sound` label. The
constructor handles the template's zero-filled label terminator when the new
label has different byte parity. Imported labels preserve their observed opaque
alignment byte until an edit crosses parity; Native itself then drops that byte
and rebuilds the zero terminator, so the bounded writer does the same.

## Playback and editing

`Edit > Action Buttons > Sound` accepts one or two selected, visible, distinct
functions. The first function supplies the mono signal or left stereo channel;
the second supplies the right channel. The independent variable is evaluated
in seconds at 44.1 kHz, finite results are clamped to `-1..1`, and undefined or
nonfinite samples become silence. Clicking a button toggles its own stream, and
Escape stops all Sound streams along with the other running actions.

Playback captures the current document when the button is activated. This
keeps the real-time source independent of subsequent editor mutation; activating
the button again uses the then-current construction. Audio-device creation is
lazy, and a missing output device produces a visible editor error instead of a
crash. The Nix development shell supplies ALSA for the Rodio playback backend.

The channel and time semantics follow the official Sound Button reference:
<https://referencecenter.dynamicgeometry.com/gsp-reference-center/sound_buttons.htm>.

Properties exposes the caption while retaining the one or two function sources.
This matches Native's two Sound-button tabs, Object and Label; there is no
separate Sound settings panel. Caption changes are transactional, undoable, and
use the shared page-local action writer on supported multi-page documents.

## Native and runtime verification

A corpus Sound fixture was edited, saved, reopened, and retained its function,
label, and anchor. A newly authored one-function button was then opened and
saved by Sketchpad 5 under Xvfb. Both the existing and new buttons were visible,
and the generated file was unchanged by Native save:

`f74e8f0b2fa7a5bec0cfb8592a25bf6d09fe0e3c425d4e865550699731c64a00`

The real port editor was also run under Xvfb with ALSA's null device. Clicking
the imported button entered the `Playing Sound button 9` state and clicking it
again reported `Stopped Sound button 9`, with no audio initialization error.

A public-API Properties edit changed the imported `music.gsp` caption from
`听到函数 f` to `Play Verified Sound`, exercising an even-to-odd UTF-8 byte
length transition. Native Properties displayed the full new caption and
described the same function `f`. Sketchpad 5 then saved all 2,898 bytes
unchanged with SHA-256
`d05070bc3fb7d2053832193c860ad18bec1fa5d4ba539a39763b959e82655488`.

Corpus coverage changes as follows:

- unsupported slots: **741 -> 723**
- fully decoded documents: **1,514 -> 1,523**
- documents with unsupported objects: **85 -> 76**
- drawing-complete documents: **1,555 -> 1,565**
- partial views: **800 -> 795**
- byte-preserving originals: **1,599 / 1,599**

The 17 Sound roots and one dependent object are restored. Embedded-tool
coverage remains **4,766 / 5,356**. The two remaining drawing code-62 groups
are the deliberately unsupported contradictory Animation packets described in
`legacy_animation_settings_analysis.md`. No standalone test case was added.
