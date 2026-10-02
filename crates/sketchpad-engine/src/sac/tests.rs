use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Query {
    Branch,
    Rounded,
    Output,
    Sample(usize),
    Samples,
    Cycle,
    Panic,
}

struct Inputs {
    values: [i64; 4],
}
type TestRuntime = Runtime<Query, usize, i64, i64>;

impl Resolver<Query, usize, i64, i64> for Inputs {
    fn input_stamp(&self, input: &usize) -> i64 {
        self.values[*input]
    }
    fn evaluate(&self, rt: &TestRuntime, key: &Query) -> Result<i64, Cycle<Query>> {
        let read = |id| {
            rt.read(self, id);
            self.values[id]
        };
        Ok(match key {
            Query::Branch => {
                if read(0) == 0 {
                    read(1)
                } else {
                    read(2)
                }
            }
            Query::Rounded => rt.query(self, Query::Branch)? / 10,
            Query::Output => rt.query(self, Query::Rounded)? * 100,
            Query::Sample(index) => read(1) + *index as i64,
            Query::Samples => (0..read(3) as usize)
                .map(|i| rt.query(self, Query::Sample(i)))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .sum(),
            Query::Cycle => rt.query(self, Query::Cycle)?,
            Query::Panic => panic!("interrupted computation"),
        })
    }
}

fn runtime() -> TestRuntime {
    Runtime::new(128.try_into().unwrap())
}

#[test]
fn actual_reads_replace_branch_edges_and_equal_results_stop_propagation() {
    let mut inputs = Inputs {
        values: [0, 12, 41, 3],
    };
    let mut rt = runtime();
    assert_eq!(rt.query(&inputs, Query::Output), Ok(100));
    assert_eq!(rt.statistics().executed, 3);
    inputs.values[2] = 52; // inactive branch
    rt.advance();
    assert_eq!(rt.query(&inputs, Query::Output), Ok(100));
    assert_eq!(rt.statistics().executed, 3);
    inputs.values[1] = 19; // branch changes, rounded result stays equal
    rt.advance();
    assert_eq!(rt.query(&inputs, Query::Output), Ok(100));
    assert_eq!(rt.statistics().executed, 5); // output did not execute
    inputs.values[0] = 1;
    rt.advance();
    assert_eq!(rt.query(&inputs, Query::Output), Ok(500));
    assert_eq!(rt.statistics().executed, 8);
    inputs.values[1] = 999; // former branch is no longer a dependency
    rt.advance();
    assert_eq!(rt.query(&inputs, Query::Output), Ok(500));
    assert_eq!(rt.statistics().executed, 8);
}

#[test]
fn stable_sample_identities_reuse_old_iterations_and_collect_removed_branches() {
    let mut inputs = Inputs {
        values: [0, 12, 41, 3],
    };
    let mut rt = runtime();
    assert_eq!(rt.query(&inputs, Query::Samples), Ok(39));
    assert_eq!(rt.statistics().executed, 4);
    inputs.values[3] = 5;
    rt.advance();
    assert_eq!(rt.query(&inputs, Query::Samples), Ok(70));
    assert_eq!(rt.statistics().executed, 7); // root + only two new samples
    inputs.values[3] = 2;
    rt.advance();
    assert_eq!(rt.query(&inputs, Query::Samples), Ok(25));
    assert_eq!(rt.statistics().executed, 8);
    rt.retain_reachable([Query::Samples]);
    assert_eq!(rt.len(), 3);
    inputs.values[3] = 3;
    rt.advance();
    assert_eq!(rt.query(&inputs, Query::Samples), Ok(39));
}

#[test]
fn forks_parallel_readers_cycles_and_panics_preserve_complete_traces() {
    let inputs = Inputs {
        values: [0, 12, 41, 3],
    };
    let rt = runtime();
    std::thread::scope(|scope| {
        for _ in 0..4 {
            scope.spawn(|| assert_eq!(rt.query(&inputs, Query::Output), Ok(100)));
        }
    });
    let mut fork = rt.clone();
    let changed = Inputs {
        values: [0, 21, 41, 3],
    };
    fork.advance();
    assert_eq!(fork.query(&changed, Query::Output), Ok(200));
    assert_eq!(rt.query(&inputs, Query::Output), Ok(100));
    assert_eq!(rt.query(&inputs, Query::Cycle), Err(Cycle(Query::Cycle)));
    assert!(std::panic::catch_unwind(|| rt.query(&inputs, Query::Panic)).is_err());
    assert_eq!(rt.query(&inputs, Query::Output), Ok(100));
}

#[test]
fn eviction_cannot_alias_an_old_child_change_stamp() {
    let mut rt = TestRuntime::new(2.try_into().unwrap());
    let mut inputs = Inputs {
        values: [0, 12, 41, 3],
    };
    assert_eq!(rt.query(&inputs, Query::Output), Ok(100));
    inputs.values[1] = 22;
    rt.advance();
    assert_eq!(rt.query(&inputs, Query::Output), Ok(200));
    assert!(rt.len() <= 2);
}

#[test]
fn exported_footprints_skip_child_validation_but_preserve_branch_changes() {
    let mut rt = runtime();
    let mut inputs = Inputs {
        values: [0, 12, 41, 3],
    };
    assert_eq!(rt.query(&inputs, Query::Output), Ok(100));
    assert!(rt.input_footprint(&Query::Output).is_some());
    rt.advance();
    assert_eq!(rt.query(&inputs, Query::Output), Ok(100));
    assert_eq!(rt.statistics().verified, 1);
    inputs.values[0] = 1;
    rt.advance();
    assert_eq!(rt.query(&inputs, Query::Output), Ok(400));
    let footprint = rt.input_footprint(&Query::Output).unwrap();
    assert!(footprint.contains(&2));
    assert!(!footprint.contains(&1));
    inputs.values[2] = 51;
    rt.advance();
    assert_eq!(rt.query(&inputs, Query::Output), Ok(500));
}

#[test]
fn footprint_cannot_mix_a_stale_parent_with_an_updated_child() {
    let mut rt = runtime();
    let mut inputs = Inputs {
        values: [0, 12, 41, 3],
    };
    assert_eq!(rt.query(&inputs, Query::Output), Ok(100));
    inputs.values[1] = 21;
    rt.advance();
    assert_eq!(rt.query(&inputs, Query::Branch), Ok(21));
    assert_eq!(rt.input_footprint(&Query::Output), None);
    assert_eq!(rt.query(&inputs, Query::Output), Ok(200));
}

#[test]
fn coalesced_traces_track_branch_inputs_and_recover_after_cycles_and_panics() {
    let mut rt = runtime();
    rt.coalesce_nested_queries(true);
    let mut inputs = Inputs {
        values: [0, 12, 41, 3],
    };
    assert_eq!(rt.query(&inputs, Query::Output), Ok(100));
    assert_eq!(
        rt.len(),
        1,
        "nested arithmetic belongs to the enclosing trace"
    );
    inputs.values[2] = 52;
    rt.advance();
    assert_eq!(rt.query(&inputs, Query::Output), Ok(100));
    assert_eq!(rt.statistics().executed, 1);
    inputs.values[0] = 1;
    rt.advance();
    assert_eq!(rt.query(&inputs, Query::Output), Ok(500));
    let footprint = rt.input_footprint(&Query::Output).unwrap();
    assert!(footprint.contains(&2));
    assert!(!footprint.contains(&1));
    assert_eq!(rt.query(&inputs, Query::Cycle), Err(Cycle(Query::Cycle)));
    assert!(std::panic::catch_unwind(|| rt.query(&inputs, Query::Panic)).is_err());
    let mut fork = rt.clone();
    inputs.values[2] = 63;
    fork.advance();
    std::thread::scope(|scope| {
        for _ in 0..4 {
            scope.spawn(|| assert_eq!(fork.query(&inputs, Query::Output), Ok(600)));
        }
    });
    assert_eq!(
        rt.query(
            &Inputs {
                values: [1, 12, 52, 3]
            },
            Query::Output
        ),
        Ok(500)
    );
}

#[test]
fn scope_snapshots_keep_demanded_dependencies_without_inherited_unrelated_memos() {
    let inputs = Inputs {
        values: [0, 12, 41, 3],
    };
    let mut rt = runtime();
    for index in 0..80 {
        rt.query(&inputs, Query::Sample(index)).unwrap();
    }
    rt.advance();
    assert_eq!(rt.query(&inputs, Query::Output), Ok(100));
    let mut scope = rt.snapshot_current();
    assert_eq!(scope.len(), 3);
    assert_eq!(
        scope.input_footprint(&Query::Output),
        rt.input_footprint(&Query::Output)
    );
    scope.advance();
    assert_eq!(
        scope.query(
            &Inputs {
                values: [0, 21, 41, 3]
            },
            Query::Output
        ),
        Ok(200)
    );
    assert_eq!(rt.query(&inputs, Query::Output), Ok(100));
}

#[test]
fn snapshots_preserve_children_when_only_the_cached_parent_is_touched() {
    let inputs = Inputs {
        values: [0, 12, 41, 3],
    };
    let rt = runtime();
    assert_eq!(rt.query(&inputs, Query::Output), Ok(100));
    let reused = rt.clone();
    assert_eq!(reused.query(&inputs, Query::Output), Ok(100));
    let snapshot = reused.snapshot_current();
    assert_eq!(snapshot.len(), 3);
    assert!(snapshot.input_footprint(&Query::Output).is_some());
}

#[test]
fn coalescing_barriers_reuse_results_and_detect_cycles_across_frames() {
    struct Barrier {
        value: i64,
        cycle: bool,
    }
    impl Resolver<u8, (), i64, i64> for Barrier {
        fn can_inline(&self, key: &u8) -> bool {
            *key != 2
        }
        fn input_stamp(&self, _: &()) -> i64 {
            self.value
        }
        fn evaluate(&self, rt: &Runtime<u8, (), i64, i64>, key: &u8) -> Result<i64, Cycle<u8>> {
            match *key {
                0 => Ok(rt.query(self, 1)? * 100),
                1 => rt.query(self, 2),
                2 if self.cycle => rt.query(self, 1),
                2 => {
                    rt.read(self, ());
                    Ok(self.value / 10)
                }
                _ => unreachable!(),
            }
        }
    }
    let mut rt = Runtime::new(16.try_into().unwrap());
    rt.coalesce_nested_queries(true);
    let mut inputs = Barrier {
        value: 12,
        cycle: false,
    };
    assert_eq!(rt.query(&inputs, 0), Ok(100));
    assert_eq!(rt.len(), 2);
    inputs.value = 19;
    rt.advance();
    assert_eq!(rt.query(&inputs, 0), Ok(100));
    assert_eq!(rt.statistics().executed, 3, "only the barrier recomputes");
    inputs.value = 21;
    rt.advance();
    assert_eq!(rt.query(&inputs, 0), Ok(200));
    let mut cyclic = Runtime::new(16.try_into().unwrap());
    cyclic.coalesce_nested_queries(true);
    inputs.cycle = true;
    assert_eq!(cyclic.query(&inputs, 0), Err(Cycle(1)));
    inputs.cycle = false;
    assert_eq!(cyclic.query(&inputs, 0), Ok(200));
}

#[test]
fn footprint_visitors_run_outside_locks_and_reject_stale_children() {
    let mut rt = runtime();
    let mut inputs = Inputs {
        values: [0, 12, 41, 3],
    };
    assert_eq!(rt.query(&inputs, Query::Output), Ok(100));
    let mut visited = Vec::new();
    assert!(rt.visit_input_footprint(&Query::Output, |input| {
        assert_eq!(rt.query(&inputs, Query::Output), Ok(100));
        visited.push(*input);
    }));
    visited.sort_unstable();
    assert_eq!(visited, vec![0, 1]);
    inputs.values[1] = 21;
    rt.advance();
    assert_eq!(rt.query(&inputs, Query::Branch), Ok(21));
    assert!(!rt.visit_input_footprint(&Query::Output, |_| panic!("stale summary")));
    assert_eq!(rt.query(&inputs, Query::Output), Ok(200));
}
