# Incremental migration: performance and verification

Measured on 2026-10-02 against commit `9104038` (custom SAC runtime with sparse
object substitutions), using the 25 files in `热研系列/3D系列` from the sibling
`gsp-rs` checkout. The source GSP files were not modified.

## Implementation

The engine now uses `incremental` 0.2.8 input variables, expert nodes and observers
for dependency scheduling and result cutoffs. The former public `sac::Runtime`
is removed. A geometry adapter discovers dependencies from tracked evaluator
reads, handles dynamic rewiring, and publishes immutable result snapshots.
Thread-local execution families reuse graphs across sampling contexts. Small
mapping and iteration calculations are coalesced into pure kernels; their
containing geometry nodes remain scheduled by Incremental. Sparse copy-on-write
object substitutions remain in place. See the
[evaluation design](SELF_ADJUSTING_COMPUTATION.md) for the boundaries and limits.

## Results

This migration is not a uniform performance improvement. In this single run,
5 of 25 update medians decreased. The toroidal coil improved from 396.570 to
253.927 ms (36.0%), and the heart surface from 480.042 to 451.775 ms (5.9%).
Projectile-motion-3 changed from 48.139 to 51.305 ms. The sphere/cap animation
regressed from 76.730 to 165.030 ms, and the cube stack from 61.820 to 90.572 ms.
Several small constructions also regressed. Shared scheduling does not eliminate
trace maintenance, input validation and graph setup costs; these need further
profiling on the regressing workloads.

Unchanged-view cache hits remain cheap (all below 0.4 ms in this run). Fully
changing heavy constructions still exceed the 16.7 ms budget for 60 FPS.
These measurements cover CPU geometry extraction, not complete rendering,
GPU work or presentation. Moving the first free point does not necessarily
change the displayed surface, so a fast update does not prove fast animation
of every object in that file.

Both release binaries ran sequentially for each file, alternating which binary
ran first. No builds, tests or profilers ran concurrently with the comparison.
Each row contains one cold extraction, the median of two unchanged extractions,
and the median of three movements of the first free point. These are single-run
observations, not confidence intervals. No sampling-quality reduction or JIT
was used. Verification and Valgrind timings are excluded from the table.

## Full comparison

All times are milliseconds; each cell is **custom SAC → Incremental**.

| Sample | First geometry | Unchanged geometry | Input update + geometry |
| --- | ---: | ---: | ---: |
| 3D正方体堆垒（未用公式解方程）.gsp | 61.297 → 100.564 | 0.256 → 0.239 | 61.820 → 90.572 |
| 也做正n棱锥表面展开.gsp | 7.387 → 18.780 | 0.173 → 0.095 | 6.717 → 18.896 |
| 仿3D.gsp | 0.307 → 0.974 | 0.022 → 0.018 | 0.083 → 0.126 |
| 再作螺线塔.gsp | 6.815 → 7.889 | 0.069 → 0.067 | 0.154 → 0.185 |
| 圆周上的点平抛运动演示.gsp | 39.110 → 41.899 | 0.082 → 0.064 | 40.399 → 44.579 |
| 圆周上的点平抛运动演示3  2022.5.2.gsp | 119.335 → 100.759 | 0.332 → 0.376 | 48.139 → 51.305 |
| 地球仪(aijun).gsp | 5.525 → 13.640 | 0.143 → 0.114 | 5.829 → 8.087 |
| 学做纬线圈 2022.5.2.gsp | 134.420 → 146.024 | 0.268 → 0.241 | 0.520 → 0.812 |
| 带状正n 边形螺线塔.gsp | 9.638 → 16.995 | 0.134 → 0.075 | 8.617 → 10.774 |
| 心形面.gsp | 490.630 → 483.998 | 0.120 → 0.118 | 480.042 → 451.775 |
| 投影.gsp | 0.308 → 2.173 | 0.044 → 0.040 | 0.150 → 0.275 |
| 无盖长方盒.gsp | 0.611 → 4.836 | 0.123 → 0.100 | 0.391 → 0.436 |
| 正n 边形螺旋线塔.gsp | 12.038 → 16.333 | 0.132 → 0.092 | 10.005 → 13.408 |
| 正四面体内阿氏球面.gsp | 4.394 → 11.988 | 0.089 → 0.117 | 2.346 → 6.137 |
| 正方体表面展开2.gsp | 0.656 → 4.905 | 0.098 → 0.056 | 0.350 → 0.523 |
| 正棱柱（棱台、棱锥、圆锥）侧面展开.gsp | 2.813 → 5.154 | 0.056 → 0.032 | 2.334 → 1.542 |
| 环形螺线管.gsp | 410.423 → 266.091 | 0.271 → 0.234 | 396.570 → 253.927 |
| 球和球缺的生成动画.gsp | 81.024 → 166.431 | 0.063 → 0.059 | 76.730 → 165.030 |
| 球极投影.gsp | 59.307 → 57.262 | 0.038 → 0.029 | 58.100 → 55.242 |
| 球面.gsp | 149.917 → 139.665 | 0.133 → 0.132 | 0.262 → 0.317 |
| 理解三视图的“三视”.gsp | 0.940 → 6.558 | 0.163 → 0.122 | 0.573 → 0.835 |
| 空间中的单摆.gsp | 21.117 → 30.515 | 0.070 → 0.057 | 11.666 → 13.131 |
| 菱形网络球.gsp | 173.300 → 165.883 | 0.104 → 0.093 | 168.607 → 164.330 |
| 螺旋迭代【3D】.gsp | 4.274 → 4.648 | 0.040 → 0.036 | 3.771 → 3.776 |
| 长方体挖圆柱20220329.gsp | 73.493 → 82.473 | 0.214 → 0.164 | 72.824 → 87.026 |

## Verification

- Workspace tests: 541 passed, including one doctest.
- Workspace Clippy, all targets, with warnings denied: passed.
- Formatting and diff whitespace checks: passed.
- All 25 samples passed two updated-view comparisons against fresh evaluation
  (50 comparisons), including preservation of the old view's geometry.
- All 25 SVG exports matched the pre-migration reference exports byte for byte.
- Adapter regressions cover generated changing DAGs against direct recursive
  evaluation, dynamic edges, simultaneous edge reversal, cache eviction,
  callback panic recovery and delayed materialization of refreshed kernel memos.
  Existing snapshot, sampling, dependency cutoff and parallel evaluation tests
  remain in the workspace suite.

Valgrind Memcheck ran projectile-motion-3 with two updates and `--verify`:

| Measurement | Incremental |
| --- | ---: |
| Allocation calls | 4,072,086 |
| Cumulative bytes allocated | 2,412,050,094 |
| Reported errors | 0 |
| Definitely / indirectly / possibly lost bytes | 0 / 0 / 0 |
| Still reachable bytes | 544 |

These allocation totals include fresh-view verification and fingerprints;
they are not peak memory or steady-state frame allocations. The memory check
covers this sample, not every construction.

## Reproduce

Build the baseline release example from commit `9104038` in a separate checkout
and preserve its executable before building the current version. Run both
sequentially with the same sample and update count:

```sh
cargo build --release --example profile_geometry --bin gsp_probe
./target/release/examples/profile_geometry /path/to/sample.gsp 3
./target/release/examples/profile_geometry /path/to/sample.gsp 2 --verify
./target/release/gsp_probe svg /path/to/sample.gsp /tmp/sample.svg
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
valgrind --tool=memcheck --leak-check=full --errors-for-leak-kinds=definite \
  --error-exitcode=1 ./target/release/examples/profile_geometry \
  /path/to/sample.gsp 2 --verify
```

`--verify` compares updated geometry against a fresh evaluated view and confirms
that the old view is unchanged. Coordinates use debug representations without
rounding. SVG comparison provides a separate renderer-level reference check.
