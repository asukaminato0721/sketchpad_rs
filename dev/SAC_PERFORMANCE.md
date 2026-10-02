# SAC performance and verification

Measured on 2026-10-02 with the 25 GSP files in `热研系列/3D系列`.
This round compares the previous optimized SAC implementation (`2fa33fb`) with
this working tree. Both release binaries ran sequentially for each sample,
without concurrent builds, profilers, other benchmarks or GUI tests.
Source GSP files were not modified.

Each row contains one cold geometry extraction, the median of two unchanged
queries, and the median of three movements of the first free point. Numbers are
single-run observations, not statistical confidence intervals. This workload
measures CPU geometry extraction, not full rendering, GPU work or presentation.
Moving the first free point does not necessarily change the displayed surface.

## Changes and profiling

`perf record` on the heart surface identified substantial time in dependency
hash-set insertion and growth, input-footprint construction, stamp copying and
dropping, and repeated construction validation. This round reduces that work:

- Mapped-element construction validity is cached across scopes sharing an object
  table. An object edit replaces the cache while preserving older snapshots.
  Queries observe the validity result as a SAC input, retaining invalidation
  when a construction becomes invalid or valid again.
- Cross-scope calls visit shared immutable input summaries. They no longer clone
  the input/stamp vector or allocate a separate vector of input IDs. Callbacks
  run outside the runtime lock.
- Engine recordings submit their already-unique inputs once, without a second
  hash-set deduplication pass. The general runtime API still deduplicates reads.
- A bounded per-thread pool reuses small recording hash sets: at most 32 sets,
  each with capacity at most 256 entries. Larger allocations are released.

The existing coordinate candidate index, per-object reads, coalesced iteration
traces, retained child computations and Arc identity checks remain in use.

## Results and limits

19 of 25 samples had a lower update median in this run. The heart surface
improved from 793.203 to 480.586 ms (39.4%), and the network sphere from 296.589
to 171.339 ms (42.2%). The solenoid improved from 416.432 to 397.097 ms (4.6%).
16 samples were below 16.7 ms for this particular CPU workload; that does not
establish 60 FPS. The heavy surfaces remain far above that budget.

Performance is not uniformly improved. The projectile-motion-3 sample regressed
from 156.397 to 173.775 ms (11.1%), with a recurring expensive third update in
longer runs as well. Disabling the recording pool did not remove this regression.
Profiling that workload shows object-table cloning as its largest individual
sampled function; avoiding construction edits during temporary sampling remains
a useful next target. Several smaller cases also regressed; see the full table.

The earlier optimized non-SAC evaluator is documented in
[3D performance](3D_PERFORMANCE.md). Its heart-surface update was about 346 ms;
this SAC evaluator still has substantial overhead relative to that historical
measurement. These results do not establish an advantage over the non-SAC
implementation for every workload.

## Reproduce

```sh
cargo build --release --example profile_geometry
./target/release/examples/profile_geometry /path/to/sample.gsp 3
./target/release/examples/profile_geometry /path/to/sample.gsp 2 --verify
perf record -F 499 -g --call-graph dwarf -o /tmp/geometry.data -- \
  ./target/release/examples/profile_geometry /path/to/sample.gsp 8
perf report --stdio --no-children -g none -i /tmp/geometry.data
valgrind --tool=memcheck --leak-check=full --errors-for-leak-kinds=definite \
  --error-exitcode=1 ./target/release/examples/profile_geometry \
  /path/to/sample.gsp 2 --verify
```

`--verify` compares incrementally evaluated geometry against a fresh evaluated
view after each movement of the first free point. It also checks that the old
view retains its original geometry. It compares debug representations without
rounding finite coordinates. Verification and profiler timings are not the
performance data in the table below.

All times are milliseconds. Each cell is **previous SAC → this round**.

| Sample | First geometry | Unchanged geometry | Input update + geometry |
| --- | ---: | ---: | ---: |
| 3D正方体堆垒（未用公式解方程）.gsp | 61.874 → 60.280 | 0.309 → 0.216 | 60.581 → 58.878 |
| 也做正n棱锥表面展开.gsp | 7.435 → 5.756 | 0.104 → 0.110 | 5.530 → 4.614 |
| 仿3D.gsp | 0.417 → 0.358 | 0.023 → 0.022 | 0.089 → 0.086 |
| 再作螺线塔.gsp | 7.132 → 7.362 | 0.062 → 0.064 | 0.161 → 0.158 |
| 圆周上的点平抛运动演示.gsp | 60.017 → 50.539 | 0.082 → 0.081 | 58.512 → 55.797 |
| 圆周上的点平抛运动演示3  2022.5.2.gsp | 209.927 → 184.770 | 0.309 → 0.316 | 156.397 → 173.775 |
| 地球仪(aijun).gsp | 6.891 → 5.952 | 0.139 → 0.137 | 6.775 → 5.654 |
| 学做纬线圈 2022.5.2.gsp | 138.909 → 142.022 | 0.233 → 0.249 | 0.468 → 0.484 |
| 带状正n 边形螺线塔.gsp | 9.689 → 11.950 | 0.095 → 0.129 | 9.653 → 10.603 |
| 心形面.gsp | 787.128 → 483.921 | 0.122 → 0.117 | 793.203 → 480.586 |
| 投影.gsp | 0.279 → 0.386 | 0.043 → 0.062 | 0.157 → 0.214 |
| 无盖长方盒.gsp | 0.940 → 0.606 | 0.171 → 0.124 | 0.539 → 0.401 |
| 正n 边形螺旋线塔.gsp | 11.866 → 10.063 | 0.145 → 0.128 | 11.613 → 9.733 |
| 正四面体内阿氏球面.gsp | 7.464 → 4.366 | 0.099 → 0.090 | 3.154 → 2.358 |
| 正方体表面展开2.gsp | 0.565 → 0.552 | 0.067 → 0.068 | 0.386 → 0.360 |
| 正棱柱（棱台、棱锥、圆锥）侧面展开.gsp | 2.484 → 2.014 | 0.038 → 0.037 | 2.015 → 1.530 |
| 环形螺线管.gsp | 418.822 → 401.251 | 0.253 → 0.268 | 416.432 → 397.097 |
| 球和球缺的生成动画.gsp | 98.658 → 80.232 | 0.068 → 0.096 | 93.183 → 76.126 |
| 球极投影.gsp | 68.985 → 63.711 | 0.041 → 0.043 | 66.758 → 58.556 |
| 球面.gsp | 171.432 → 148.595 | 0.148 → 0.189 | 0.251 → 0.318 |
| 理解三视图的“三视”.gsp | 0.816 → 1.117 | 0.154 → 0.229 | 0.590 → 0.816 |
| 空间中的单摆.gsp | 24.508 → 22.544 | 0.075 → 0.071 | 13.359 → 13.211 |
| 菱形网络球.gsp | 303.849 → 181.977 | 0.116 → 0.106 | 296.589 → 171.339 |
| 螺旋迭代【3D】.gsp | 7.319 → 4.025 | 0.050 → 0.040 | 4.391 → 3.722 |
| 长方体挖圆柱20220329.gsp | 82.034 → 80.567 | 0.239 → 0.205 | 76.982 → 73.864 |

## Verification

- Full workspace: 543 tests passed, including one doctest.
- Workspace Clippy, all targets, with warnings denied: passed.
- Formatting and diff whitespace checks: passed.
- All 25 SVG exports matched the pre-SAC reference exports byte for byte.
- All 25 samples passed two update comparisons with fresh evaluation
  (50 comparisons), including old-snapshot isolation.
- Added regressions cover mapping-validity invalidation and snapshot isolation,
  recording cleanup after nested unwinding, and footprint callbacks that reenter
  the runtime and reject stale child computations.
- Valgrind 3.25.1 Memcheck on `螺旋迭代【3D】.gsp`, with two updates and fresh-view
  verification: zero reported errors, zero definitely or indirectly lost bytes.
  It reported 48 possibly lost bytes allocated by Rust's thread initialization
  and 544 still-reachable bytes. This is a check of one sample, not the whole
  corpus. The Arch debuginfod service supplied the required glibc debug symbols.

The implementation remains an interpreted SAC runtime. No JIT or tessellation
quality reduction is used for these results.
