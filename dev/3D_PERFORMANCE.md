# 3D sample performance

Measured on 2026-10-02 with the 25 files in
`gsp-rs/tests/Samples/热研系列/3D系列`. Source sample files were not modified.

## Changes

- `ObjectStore` shares an `Arc<Vec<Object>>` between evaluation scopes and undo
  snapshots. All mutation guards use `Arc::make_mut`, preserving independent
  edits and old views while avoiding repeated construction-table copies.
- Derived iteration geometry and mapped curves/surfaces use a per-scope LRU
  cache (64 successful queries). Keys include object ID, geometry query, output
  type and sampling resolution. Payloads are immutable and shared across cloned
  scopes; dependency invalidation removes affected entries in the new scope.
  Cheap non-matching geometry queries do not evict useful results.
- Mapped elements materialize their shared intermediate values. Iteration seed
  advancement and output generation use separate dependency slices. Numeric seed
  boundaries are invalidated once before replay, instead of traversing the full
  dependency graph for every state.
- Structural transform validation survives value-only substitutions. Nested
  scopes inherit it only when they share the exact immutable object table.

## Reproduce

```sh
cargo build --release --example profile_geometry
./target/release/examples/profile_geometry /path/to/sample.gsp 3
```

The example reports file parsing, initial evaluation, first geometry extraction,
unchanged repeats, and evaluation plus geometry after moving the first free
point. Sampling matches the renderer: 384 curve samples, 96 for mapped arcs,
and 128 for ordinary arcs. Initial extraction also reports objects costing
more than 0.5 ms.

These are CPU geometry measurements, not full frame or GPU timings. The update
is one particular point moving by a few world units, not every possible drag or
animation. An unrelated point can leave most cached geometry valid. The table
contains one cold measurement and medians of two warm queries / three updates;
small sub-millisecond differences are noisy. Final timing ran without concurrent
compilation or other benchmark processes.

## Results

All times are milliseconds; each cell is **before → after**.

| Sample | First geometry | Unchanged geometry | Input update + geometry |
| --- | ---: | ---: | ---: |
| 3D正方体堆垒（未用公式解方程）.gsp | 75.487 → 27.945 | 74.294 → 0.082 | 73.175 → 27.935 |
| 也做正n棱锥表面展开.gsp | 8.734 → 3.645 | 8.576 → 0.042 | 8.858 → 3.617 |
| 仿3D.gsp | 0.238 → 0.242 | 0.151 → 0.007 | 0.167 → 0.320 |
| 再作螺线塔.gsp | 12.424 → 3.005 | 12.438 → 0.025 | 13.036 → 0.058 |
| 圆周上的点平抛运动演示.gsp | 5.834 → 4.487 | 0.011 → 0.015 | 20.688 → 15.217 |
| 圆周上的点平抛运动演示3  2022.5.2.gsp | 31.273 → 22.348 | 0.083 → 0.094 | 100.445 → 76.485 |
| 地球仪(aijun).gsp | 8.519 → 2.925 | 5.576 → 0.090 | 8.557 → 2.900 |
| 学做纬线圈 2022.5.2.gsp | 16.875 → 14.602 | 14.245 → 0.084 | 14.674 → 0.291 |
| 带状正n 边形螺线塔.gsp | 24.407 → 5.377 | 24.772 → 0.026 | 25.371 → 3.809 |
| 心形面.gsp | 1532.346 → 350.158 | 1573.203 → 0.037 | 1535.498 → 345.611 |
| 投影.gsp | 0.017 → 0.029 | 0.009 → 0.013 | 0.081 → 0.078 |
| 无盖长方盒.gsp | 0.040 → 0.050 | 0.016 → 0.036 | 0.284 → 0.222 |
| 正n 边形螺旋线塔.gsp | 72.529 → 6.809 | 82.633 → 0.032 | 44.650 → 4.790 |
| 正四面体内阿氏球面.gsp | 4.770 → 3.375 | 1.936 → 0.040 | 5.382 → 2.945 |
| 正方体表面展开2.gsp | 0.040 → 0.038 | 0.022 → 0.024 | 0.459 → 0.213 |
| 正棱柱（棱台、棱锥、圆锥）侧面展开.gsp | 9.360 → 0.838 | 8.999 → 0.010 | 9.903 → 0.736 |
| 环形螺线管.gsp | 443.530 → 319.356 | 0.061 → 0.070 | 371.464 → 319.995 |
| 球和球缺的生成动画.gsp | 49.411 → 38.469 | 0.183 → 0.016 | 52.029 → 37.065 |
| 球极投影.gsp | 181.150 → 20.643 | 183.690 → 0.016 | 182.094 → 20.557 |
| 球面.gsp | 140.758 → 34.679 | 138.651 → 0.049 | 138.745 → 0.123 |
| 理解三视图的“三视”.gsp | 0.048 → 0.070 | 0.035 → 0.045 | 0.321 → 0.338 |
| 空间中的单摆.gsp | 8.366 → 3.550 | 0.009 → 0.013 | 2.873 → 2.953 |
| 菱形网络球.gsp | 730.512 → 202.135 | 754.149 → 0.035 | 712.576 → 200.697 |
| 螺旋迭代【3D】.gsp | 9.336 → 1.747 | 11.698 → 0.018 | 10.283 → 1.710 |
| 长方体挖圆柱20220329.gsp | 100.881 → 27.577 | 72.014 → 0.048 | 110.937 → 25.312 |

The heart surface, network sphere and solenoid still exceed the 16.7 ms budget
when their geometry changes. This change substantially improves reuse and
several live constructions, but does not establish 60 FPS for all samples.
Remaining work includes reducing nested mapping validation/replay and the
solenoid's nested locus sampling; unchanged-cache timings must not be presented
as animation FPS.

## Correctness

All 25 SVG exports are byte-for-byte identical to the exports made before these
changes, with unchanged sampling counts. Regression coverage compares cached
collections against uncached evaluation after dragging, changing iteration
depth, undo, and switching sampling resolutions. Older evaluated views retain
their original results. Existing tests also cover nested mapping scope rules,
undefined samples, graph rewiring and interrupted edits.

`cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`,
and `cargo fmt --all -- --check` pass. The release desktop application opens the
heart surface, solenoid and larger projectile sample under Xvfb and redraws after
zoom input. This is a GUI smoke check, not a frame-rate measurement.
