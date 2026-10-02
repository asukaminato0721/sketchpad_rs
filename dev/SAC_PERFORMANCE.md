# SAC performance and verification

Measured on 2026-10-02 with the 25 GSP files in `热研系列/3D系列`.
This round compares commit `2710d7a` with sparse sampling substitutions in the
working tree. Both release binaries ran sequentially for each sample, without
concurrent builds, profilers, other benchmarks or GUI tests. Source GSP files
were not modified.

Each table row contains one cold geometry extraction, the median of two unchanged
queries, and the median of three movements of the first free point. These are
single-run observations, not statistical confidence intervals. The workload
measures CPU geometry extraction, not full rendering, GPU work or presentation.
Moving the first free point does not necessarily change the displayed surface.

## Change

Geometric locus sampling and curve-intersection refinement replace a driver's
constraint parameters. Previously, those replacements called `ObjectStore::edit`,
which could clone the entire shared object vector and its revision vector.
Nested sampling snapshots repeatedly made these vectors shared again.

Sampling now stores sparse replacements over the immutable base object table.
The replacement map and objects use `Arc::make_mut` for copy-on-write isolation.
Each replacement carries its own SAC revision token. Lookup, indexing and
iteration read replacements directly; direct parameter measurements therefore
continue to see sampled circle, arc and polygon parameters, as well as positions.
Graph reconnection, validation invalidation and change history remain shared with
the ordinary edit path. Old and nested snapshots retain their own values.

A caller requesting a contiguous slice lazily materializes the merged table.
Ordinary mutable edits also materialize it, preserving replacement revision
tokens. The optimization does not remove full copies from those paths or from
structural edits that require a new dependency graph.

## Results and profiling

In the three-update comparison, projectile-motion-3 improved from 200.002 to
46.889 ms (76.6%), and the other projectile-motion sample from 63.411 to 39.187 ms
(38.2%). Separate 24-update runs, repeated in opposite binary order, measured
190.828–194.601 ms before and 48.603–48.698 ms after for projectile-motion-3.

The heart surface (488.312 → 495.263 ms) and network sphere
(171.899 → 172.028 ms) are essentially unchanged in this run. This optimization
targets temporary construction edits; it does not remove their numerical and
SAC trace costs. Heavy samples, including projectile-motion-3, still exceed
16.7 ms. These CPU measurements do not establish 60 FPS.

13 of 25 samples had a lower update median in the single-run table. Several
small cases fluctuated. Follow-up 24-update runs in opposite order did not
reproduce the table's large spiral-iteration and cube-stack regressions:
spiral medians were 3.620–3.684 ms before and 3.608–3.698 ms after; cube-stack
medians were 60.348–61.710 ms before and 60.462–61.817 ms after. The full table
retains the original observations rather than substituting follow-up numbers.

`perf record -F 999 -g --call-graph dwarf`, with 30 updates of projectile-motion-3,
reduced the exclusive sampled share of the whole-object-vector copy routine
from 20.95% to 0.12%. These are profile shares, not elapsed-time speedups.

Valgrind Memcheck on that same sample with two updates and `--verify` measured:

| Measurement | Before | After |
| --- | ---: | ---: |
| Allocation calls | 8,503,099 | 4,524,946 |
| Cumulative bytes allocated | 6,650,747,934 | 3,489,794,461 |
| Reported errors | 0 | 0 |
| Definitely / indirectly lost bytes | 0 / 0 | 0 / 0 |

Cumulative allocation fell by 47.5%. These totals include fresh-view verification
and its geometry fingerprints; they are not peak memory or steady-state frame
allocations. Both binaries reported the same 48 possibly lost bytes from Rust
thread initialization and 544 still-reachable bytes.

The earlier non-SAC evaluator is documented in
[3D performance](3D_PERFORMANCE.md). Its historical heart-surface update was
about 346 ms, below the current SAC measurement. This round does not claim a
uniform advantage over that evaluator.

## Reproduce

```sh
cargo build --release --example profile_geometry
./target/release/examples/profile_geometry /path/to/sample.gsp 3
./target/release/examples/profile_geometry /path/to/sample.gsp 2 --verify
perf record -F 999 -g --call-graph dwarf -o /tmp/geometry.data -- \
  ./target/release/examples/profile_geometry /path/to/sample.gsp 30
perf report --stdio --no-children -g none -i /tmp/geometry.data
valgrind --tool=memcheck --leak-check=full --errors-for-leak-kinds=definite \
  --error-exitcode=1 ./target/release/examples/profile_geometry \
  /path/to/sample.gsp 2 --verify
```

`--verify` compares incrementally evaluated geometry against a fresh evaluated
view after each movement. It also checks that the old view retains its original
geometry. Coordinates are compared through debug representations without rounding.
Verification and profiler timings are not the performance data in the table.

## Full comparison

All times are milliseconds. Each cell is **previous SAC → sparse substitutions**.

| Sample | First geometry | Unchanged geometry | Input update + geometry |
| --- | ---: | ---: | ---: |
| 3D正方体堆垒（未用公式解方程）.gsp | 60.016 → 64.784 | 0.267 → 0.247 | 59.164 → 65.102 |
| 也做正n棱锥表面展开.gsp | 7.462 → 5.818 | 0.289 → 0.111 | 4.618 → 4.985 |
| 仿3D.gsp | 0.351 → 0.255 | 0.036 → 0.016 | 0.066 → 0.060 |
| 再作螺线塔.gsp | 8.167 → 5.896 | 0.096 → 0.089 | 0.229 → 0.170 |
| 圆周上的点平抛运动演示.gsp | 56.249 → 34.737 | 0.088 → 0.080 | 63.411 → 39.187 |
| 圆周上的点平抛运动演示3  2022.5.2.gsp | 199.140 → 117.715 | 0.340 → 0.337 | 200.002 → 46.889 |
| 地球仪(aijun).gsp | 6.531 → 5.612 | 0.184 → 0.148 | 6.813 → 5.575 |
| 学做纬线圈 2022.5.2.gsp | 133.540 → 130.199 | 0.286 → 0.271 | 0.553 → 0.554 |
| 带状正n 边形螺线塔.gsp | 8.379 → 10.882 | 0.088 → 0.098 | 7.910 → 8.396 |
| 心形面.gsp | 489.535 → 514.309 | 0.151 → 0.115 | 488.312 → 495.263 |
| 投影.gsp | 0.730 → 0.264 | 0.064 → 0.044 | 0.154 → 0.151 |
| 无盖长方盒.gsp | 0.846 → 0.604 | 0.179 → 0.120 | 0.531 → 0.373 |
| 正n 边形螺旋线塔.gsp | 10.538 → 14.854 | 0.114 → 0.221 | 9.780 → 9.528 |
| 正四面体内阿氏球面.gsp | 6.458 → 4.688 | 0.136 → 0.089 | 2.309 → 2.503 |
| 正方体表面展开2.gsp | 0.526 → 0.488 | 0.069 → 0.069 | 0.349 → 0.342 |
| 正棱柱（棱台、棱锥、圆锥）侧面展开.gsp | 1.942 → 3.587 | 0.038 → 0.039 | 1.665 → 1.708 |
| 环形螺线管.gsp | 389.915 → 391.998 | 0.284 → 0.503 | 383.957 → 382.687 |
| 球和球缺的生成动画.gsp | 91.876 → 81.898 | 0.066 → 0.071 | 78.851 → 78.984 |
| 球极投影.gsp | 60.125 → 59.718 | 0.064 → 0.042 | 60.546 → 57.247 |
| 球面.gsp | 149.745 → 150.381 | 0.217 → 0.437 | 0.255 → 0.270 |
| 理解三视图的“三视”.gsp | 1.053 → 0.860 | 0.221 → 0.157 | 0.842 → 0.590 |
| 空间中的单摆.gsp | 22.724 → 22.447 | 0.070 → 0.069 | 15.679 → 11.631 |
| 菱形网络球.gsp | 176.647 → 182.255 | 0.106 → 0.143 | 171.899 → 172.028 |
| 螺旋迭代【3D】.gsp | 4.537 → 5.949 | 0.040 → 0.089 | 3.692 → 5.545 |
| 长方体挖圆柱20220329.gsp | 75.136 → 78.358 | 0.221 → 0.228 | 72.429 → 74.927 |

## Verification

- Full workspace: 546 tests passed, including one doctest.
- Workspace Clippy, all targets, with warnings denied: passed.
- Formatting and diff whitespace checks: passed.
- All 25 SVG exports matched the pre-SAC reference exports byte for byte.
- All 25 samples passed two update comparisons with fresh evaluation
  (50 comparisons), including old-snapshot isolation.
- New regressions cover sparse reads through indexing, iteration and slices;
  shared storage and unchanged-object revision preservation; materialization
  followed by ordinary edits, insertion and bulk mutation; graph rewiring and
  cycle detection; and point-parameter measurements in nested sampling scopes.
- Valgrind 3.25.1 Memcheck: results and scope are reported above.

The implementation remains an interpreted SAC runtime. No JIT or tessellation
quality reduction is used for these results.
