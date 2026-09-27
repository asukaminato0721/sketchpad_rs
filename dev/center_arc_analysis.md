# Center arcs

2026-09-27, checkpoint 34. Copying and custom tools remain deferred.

## Native records and semantics

Native code80 has a 28-byte header, class12/subtype67, and three point
references: center, radius/start point, end direction. The new basic
`Kind::CenterArc` describes this construction; it does not add transform/shape
combinations. Existing matrix sequences and numeric expressions remain shared.

Four schema controls replace the opcode of the existing three-point arc writer
with code80 while preserving the common class/subtype and record order. Native
opens and saves each exactly; screenshots were inspected. A 60-pixel radius
produces 90 degrees and 2.49 centimeters for both an on-circle endpoint and a
farther point in the same radial direction. The opposite end direction produces
270 degrees and 7.48 centimeters. Coincident start/end references produce an
undefined arc, constrained point, segment and measurements. These controls are
schema/semantic evidence, separate from the production writer proof.

The evaluator uses the start point's radius and the end point's direction.
It retains the positive circular sweep and rejects zero radius or zero sweep.
The original clock corpus contains a 0.001-degree center arc; its regression
checks the small sweep, radius and exact preservation of all original records.

## Implementation and actual writer proof

Decode/encode, dependency validation, stroke metadata, sampling, picking,
construction commands, measurements, constrained points and parameter/literal
path points share the existing arc interfaces. A Point Value record on this
family has the circle-arc layout, without a stored normalized-parameter cache.
It can therefore be serialized when the geometry is undefined. Validation
permits the structurally valid undefined graph while retaining parent checks.

The actual API writer creates thirteen objects: three free points, center arc,
angle, length, constrained midpoint and segment, parameter, parameter point,
literal point, point value and angle-derived calculation. Moving the end point
changes the angle to270 and the calculation to1.50; the point parameter is1.00.
Native opens/saves this 3050-byte output exactly and displays those values.
A second API output collapses the sweep; Native opens/saves its 3050 bytes
exactly, leaving only the free points and parameter visible. Both independent
comparisons against untouched snapshots pass. Their retained fixtures are exact
regression goldens; the quarter-arc writer also equals its Native fixture.

Tests cover original import, radial projection, major/small sweeps, undefined
states, live changes, undo/redo, measurements, Point Value, calculations,
multi-page local references and editor commands. Pixel-identical rendering is
outside this work's scope.

## Coverage

Checkpoint34 recovers4941 drawing slots, including361 calculations, plus26
prototype slots. Fully decoded documents rise from86 to89, and all1599 original
files retain exact round trips. The remaining73059 unsupported slots mean this
is an incremental family implementation, not full command parity.


Completed scoped gates:613 library,109 editor,34 numeric-import and4 coverage
tests pass. The unfiltered numeric-import run exposes two existing clipboard/copy
failures; those lie outside the user's requested scope. The final whole-corpus
scan has0 failed documents,1599 exact round trips and73059 unsupported slots.
