# Tool folder workflow audit

This audit separates existing proof from proposed behavior. The current port
lists, creates, and invokes tools embedded in the current document. Folder tool
registration and invocation require separate evidence; recognizing an embedded
prototype does not establish folder discovery or namespace persistence.

## Existing local evidence

The Native custom-tool flyout capture
`tests/fixtures/embedded_tool_simple_created_writer_native_verified_menu.png`
shows the newly registered tool separated from the commands above and below it.
Its Chinese command labels render as missing glyphs, so their exact text cannot
be inferred from that image alone. UTF-16 resources in the local `GSP5Chs.exe`
independently identify Choose Tool Folder at `0x3e9ad8`, Forget Tool Folder at
`0x3e9aea`, and separate current-document, other-document, and tool-folder
sections around `0x3e9b76`. No online source was needed for these local resources.

Folder feedback strings around `0x3f815e` and `0x3f81f4` distinguish document
counts from tool counts and state that newly added documents become available
at the next application launch. This supports a cached catalog, rather than
automatic directory scans on every UI frame. It does not establish discovery
depth or ordering. The exit warning at `0x3ef33c` says folder-tool edits are not
saved automatically; it directs users to edit the source document or use
Document Options to copy an edited tool into the current document. Invocation
embedding still requires a saved-file control; that warning alone is insufficient.

The centrally locked, corrected-environment flyout probe succeeds and saves its
untouched point-only input byte-identically at 1,836 bytes. Its folder command
appears at y505 when no embedded tools are present, versus y547 in the earlier
created-tool capture. The delayed chooser control exposes the standard Win32 folder dialog
(class `#32770`, title “选择工具文件夹”). Its Look-in combo is control 1137,
Select is button 1, Cancel is button 2, and the example-document checkbox is
control 1566. The dialog initially shows `/tmp`, even when the source drawing
lives inside the audit folder. X11 focus and painting do not reliably follow
the dialog. A pointerless control-message guard reads Look-in and cancels unless
it exactly matches the uniquely named audit folder; its completed control read
“tmp” and cancelled. A separate subsequent flyout matched the original no-folder
menu, so those attempts did not register a persistent folder. The disposable
folder contains Square, simple-tool, and ordinary drawing files, plus one nested
tool file. Its contents have not yet been registered successfully. The example
creation checkbox has not been activated.
Retained captures are `dev/tool_folder_native_menu.png` and
`dev/tool_folder_native_chooser.png`. A subsequent pointerless ListView control
reported 244 entries and selected index 228, but navigated to
`sketchpad_rs_wine` rather than the uniquely named owned folder. The exact-name
guard cancelled registration; its source again saved unchanged at 1,836 bytes.
These captures establish the menu and chooser controls, not successful folder
loading or tool discovery semantics.

`../gsp-rs/src/format.rs` confirms the distinction between declared drawing pages
and subsequent tool construction namespaces. It supplies no folder discovery,
menu ordering, or preference persistence implementation. Searches of local
paired HTML/text sources and existing analysis documents found no tool-folder
workflow description.

The port's existing `rfd::FileDialog` calls cancel without changing the document.
Opening a sketch uses `replace_document`, which reconstructs the entire editor
and currently preserves only the clipboard. A session-wide tool-folder catalog
would therefore need explicit preservation across Open/New/Close, or ownership
outside the replaceable document editor. Tool-folder changes must not be modeled
as drawing edits merely to retain them across document replacement.

The established embedded runtime has independent prototype/font namespaces,
ordered input declarations, active-page point binding, live dependency remapping,
visible-output selection, and clone/save/reparse validation with one undo step.
The editor click workflow reuses an existing point or creates the same constrained
or Free point as the Point tool, checking a saved clone before committing an input.
These paths can be shared by embedded and folder tools once the external-source
API establishes its own import/persistence semantics.

## Concrete proposed implementation

Choose Tool Folder opens `rfd::FileDialog::pick_folder`. Cancel preserves the
current catalog and active input binding. Folder loading builds a replacement
catalog before assignment; an unreadable directory leaves the previous catalog
available and reports the error. A successful selection preserves ordinary
application state and does not add a drawing undo step.

The catalog keeps canonical source paths and per-file ordered tool indices,
display names, declared inputs, immutable parsed source documents, and per-file
errors. Names alone must never identify a tool: duplicate names in different
files or in the destination's embedded namespace remain distinct. The final
Native-proven grouping, discovery depth, extensions, and ordering will determine
the menu presentation; those details are not yet accepted assumptions.

File scanning/parsing occurs when selecting or refreshing a folder, not on each
UI frame. Cache identity includes canonical path and file revision; invocation
uses a stable source snapshot or explicitly reloads and verifies its revision.
A failed or changed file cannot silently invoke another tool at the same menu
index. Bad files must not prevent using independent valid tools unless Native
establishes that folder loading is atomic across all entries.

The Custom Tools menu and toolbox should retain a clearly separated current
sketch section and folder entries, with the existing Apply to Selected Inputs and
Choose Inputs by Clicking commands. The active binding identifies both source
file and tool index, rather than reusing an embedded index in the current sketch.
Expected inputs are inspected in the chosen prototype's local namespace. Its
runtime eligibility is checked before creating click-input points.

The settled API should expose declared input categories and eligibility to the
editor, rather than requiring another UI-specific placeholder-kind whitelist.
The separate Given-category Native audit is extending beyond the current Point
scope; a neutral prototype Segment used to represent a Path input must not be
mistaken for a point slot or narrowed to a literal Segment-only requirement.
Blank-click point creation applies only to declared point categories. Other
categories use the runtime's independently proved existing-object capability
predicate. Folder listing must not broaden these categories by itself.

Forget Tool Folder drops the session catalog and cancels an active folder-tool
binding. Existing input points and completed drawing geometry remain. Whether
completed invocations also retain an embedded copy of the prototype must follow
the Native save/reopen control. Forget must not remove already saved definitions
or geometry merely because they came from the forgotten folder.

External invocation belongs in a Document transaction: materialize geometry with
ordered live destination references; copy a prototype into the destination only
if independently proved; preserve raw source fields and local fonts correctly;
validate saving/reparsing; then commit one undo step. The editor supplies inputs
and updates visible-output selection. It must not implement a temporary sketch
swap, clipboard offset, source-file rewrite, or UI-side reference renumbering.

## Native controls required before production changes

The planned controls distinguish registration from use. They capture the flyout
before/after Choose Tool Folder, checks ordinary drawings and nested discovery,
invokes a named folder tool into a separate point-only drawing, then saves and
compares scene counts and `03ea`/`04b4` records. Forget and reopening that saved
drawing establish whether used definitions are embedded independently of folder
registration. Folder-only dirty state, undo behavior, name collisions, source
changes after catalog loading, and restart persistence remain separate checks.

No production code or shared Wine preference changes have been made outside the
central `dev/wine_roundtrip.sh` queue. The API and folder semantics remain pending
until these controls settle. The first centrally locked menu-label probe exited
before opening a window with `could not load kernel32.dll, status c0000135`;
that environment failure establishes no folder or file-format behavior.
