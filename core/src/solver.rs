use {
    crate::puzzle::{Vial, pour},
    std::collections::HashSet,
};

pub fn solve<const N: usize, const T: usize, const C: usize>(
    mut init: [Vial<N>; T],
) -> Option<usize> {
    init.sort_unstable();
    let init = annotate_all(init);
    let b = init.iter().fold(0, |acc, v| acc + v.blocks);
    let max_x = b - C;
    let mut y = 0;
    let init_set: HashSet<[AnnotatedVial<N>; T]> = core::iter::once(init).collect();
    let zero_x: Vec<HashSet<[AnnotatedVial<N>; T]>> = core::iter::once(init_set)
        .chain(core::iter::repeat_n(HashSet::new(), b))
        .collect();
    let mut states = vec![zero_x];
    loop {
        let mut new_states = 0;
        let mut y_plus_1 = Vec::new();
        for _ in 0..=max_x {
            y_plus_1.push(HashSet::new());
        }
        for (x, y_plus_1_at_x) in y_plus_1.iter_mut().enumerate().take(max_x) {
            let Ok([y_x, y_x_plus_1]) = states[y].get_disjoint_mut([x, x + 1]) else {
                continue;
            };
            for s in y_x.iter() {
                for (p, is_decreased_blocks) in moves(s) {
                    if is_decreased_blocks {
                        if y_x_plus_1.insert(p) {
                            new_states += 1;
                        }
                    } else {
                        if y_plus_1_at_x.insert(p) {
                            new_states += 1;
                        }
                    }
                }
            }
        }
        states.push(y_plus_1);

        if !states[y][max_x].is_empty() {
            return Some(y + max_x);
        }

        // If no new states have been found then there is no solution.
        if new_states == 0 {
            return None;
        }
        y += 1;
    }
}

fn moves<const N: usize, const T: usize>(
    state: &[AnnotatedVial<N>; T],
) -> impl Iterator<Item = ([AnnotatedVial<N>; T], bool)> {
    state
        .iter()
        .enumerate()
        .filter_map(|(i, src)| src.lead_colour.map(|c| (i, c, src)))
        .flat_map(move |(i, src_c, src)| {
            state.iter().enumerate().filter_map(move |(j, dst)| {
                if i == j {
                    None
                } else {
                    match dst.lead_colour {
                        // Destination is empty
                        None => {
                            if src.blocks == 1 {
                                // It's pointless to move a single block into an empty vial.
                                None
                            } else {
                                // More than one colour in source.
                                let mut item = *state;
                                let [vial1, vial2] = item.get_disjoint_mut([i, j]).ok()?;
                                pour(&mut vial1.vial, &mut vial2.vial);
                                vial1.blocks -= 1;
                                vial1.lead_colour = vial1.vial.iter().find(|c| **c > 0).copied();
                                vial2.blocks += 1;
                                vial2.lead_colour = Some(src_c);
                                item.sort_unstable();
                                Some((item, false))
                            }
                        }
                        // Destination is non-empty
                        Some(dst_c) => {
                            let block_size = src
                                .vial
                                .iter()
                                .skip_while(|c| **c == 0)
                                .take_while(|c| **c == src_c)
                                .count();
                            let dst_empty_size = dst.vial.iter().take_while(|c| **c == 0).count();

                            if block_size <= dst_empty_size {
                                // All source blocks fit into the destination container.
                                let mut item = *state;
                                let [vial1, vial2] = item.get_disjoint_mut([i, j]).ok()?;
                                pour(&mut vial1.vial, &mut vial2.vial);
                                vial1.blocks -= 1;
                                vial1.lead_colour = vial1.vial.iter().find(|c| **c > 0).copied();
                                if dst_c == src_c {
                                    // Number of blocks reduced by combining src and dst
                                    item.sort_unstable();
                                    Some((item, true))
                                } else {
                                    // Added a different colour on top; total blocks unchanged
                                    vial2.blocks += 1;
                                    vial2.lead_colour = Some(src_c);
                                    item.sort_unstable();
                                    Some((item, false))
                                }
                            } else {
                                // Not all source blocks fit into the destination container.
                                if dst_c == src_c {
                                    let mut item = *state;
                                    let [vial1, vial2] = item.get_disjoint_mut([i, j]).ok()?;
                                    pour(&mut vial1.vial, &mut vial2.vial);
                                    item.sort_unstable();
                                    // Annotations remain unchanged
                                    Some((item, false))
                                } else {
                                    // Do not split an existing block if it creates a new block in dst
                                    None
                                }
                            }
                        }
                    }
                }
            })
        })
}

fn annotate<const N: usize>(v: Vial<N>) -> AnnotatedVial<N> {
    let Some(start) = v.iter().position(|c| c > &0) else {
        return AnnotatedVial::new(v, 0, None);
    };
    let mut blocks = 1;
    let lead_colour = v.get(start);
    let mut c = lead_colour;
    for i in start..N {
        let new_c = v.get(i);
        if c != new_c {
            blocks += 1;
            c = new_c;
        }
    }
    AnnotatedVial::new(v, blocks, Some(lead_colour))
}

fn annotate_all<const N: usize, const T: usize>(state: [Vial<N>; T]) -> [AnnotatedVial<N>; T] {
    core::array::from_fn(|i| annotate(state[i]))
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
struct AnnotatedVial<const N: usize> {
    vial: Vial<N>,
    blocks: usize,
    lead_colour: Option<u8>,
}

impl<const N: usize> AnnotatedVial<N> {
    fn new(vial: Vial<N>, blocks: usize, lead_colour: Option<u8>) -> Self {
        Self {
            vial,
            blocks,
            lead_colour,
        }
    }
}

#[test]
fn test_solve() {
    let example = [
        Vial::new([1; 5]),
        Vial::new([2; 5]),
        Vial::new([3; 5]),
        Vial::new([4; 5]),
        Vial::new([0; 5]),
        Vial::new([0; 5]),
    ];
    let steps = solve::<5, 6, 4>(example).unwrap();
    assert_eq!(steps, 0);

    let example = [
        Vial::new([0, 0, 1, 1, 1]),
        Vial::new([2; 5]),
        Vial::new([3; 5]),
        Vial::new([4; 5]),
        Vial::new([0; 5]),
        Vial::new([0, 0, 0, 1, 1]),
    ];
    let steps = solve::<5, 6, 4>(example).unwrap();
    assert_eq!(steps, 1);

    let example = [
        Vial::new([0, 4, 4, 4, 3]),
        Vial::new([0, 1, 1, 1, 1]),
        Vial::new([3, 3, 4, 3, 3]),
        Vial::new([0, 0, 0, 1, 4]),
        Vial::new([2, 2, 2, 2, 2]),
        Vial::new([0, 0, 0, 0, 0]),
    ];
    let steps = solve::<5, 6, 4>(example).unwrap();
    assert_eq!(steps, 5);

    let example = [
        Vial::new([4, 3, 4, 1, 1]),
        Vial::new([2, 1, 2, 3, 4]),
        Vial::new([2, 1, 4, 2, 3]),
        Vial::new([2, 3, 3, 1, 4]),
        Vial::new([0; 5]),
        Vial::new([0; 5]),
    ];
    let steps = solve::<5, 6, 4>(example).unwrap();
    assert_eq!(steps, 15);
}

#[test]
fn test_annotate() {
    let example = [
        Vial::new([4, 3, 4, 1, 1]),
        Vial::new([2, 1, 2, 3, 4]),
        Vial::new([2, 1, 4, 2, 3]),
        Vial::new([2, 3, 3, 1, 4]),
        Vial::new([0; 5]),
        Vial::new([0; 5]),
    ];
    assert_eq!(
        annotate(example[0]),
        AnnotatedVial::new(example[0], 4, Some(4))
    );
    assert_eq!(
        annotate(example[1]),
        AnnotatedVial::new(example[1], 5, Some(2))
    );
    assert_eq!(
        annotate(example[2]),
        AnnotatedVial::new(example[2], 5, Some(2))
    );
    assert_eq!(
        annotate(example[3]),
        AnnotatedVial::new(example[3], 4, Some(2))
    );
    assert_eq!(
        annotate(example[4]),
        AnnotatedVial::new(example[4], 0, None)
    );
    assert_eq!(
        annotate(example[5]),
        AnnotatedVial::new(example[5], 0, None)
    );

    let example = [
        Vial::new([1; 5]),
        Vial::new([2; 5]),
        Vial::new([3; 5]),
        Vial::new([4; 5]),
        Vial::new([0; 5]),
        Vial::new([0; 5]),
    ];
    assert_eq!(
        annotate(example[0]),
        AnnotatedVial::new(example[0], 1, Some(1))
    );
    assert_eq!(
        annotate(example[1]),
        AnnotatedVial::new(example[1], 1, Some(2))
    );
    assert_eq!(
        annotate(example[2]),
        AnnotatedVial::new(example[2], 1, Some(3))
    );
    assert_eq!(
        annotate(example[3]),
        AnnotatedVial::new(example[3], 1, Some(4))
    );
    assert_eq!(
        annotate(example[4]),
        AnnotatedVial::new(example[4], 0, None)
    );
    assert_eq!(
        annotate(example[5]),
        AnnotatedVial::new(example[5], 0, None)
    );

    let example = [
        Vial::new([0, 0, 3, 4, 2]),
        Vial::new([4, 4, 4, 1, 4]),
        Vial::new([0, 1, 3, 2, 1]),
        Vial::new([2, 2, 2, 1, 1]),
        Vial::new([0, 0, 3, 3, 3]),
        Vial::new([0; 5]),
    ];
    assert_eq!(
        annotate(example[0]),
        AnnotatedVial::new(example[0], 3, Some(3))
    );
    assert_eq!(
        annotate(example[1]),
        AnnotatedVial::new(example[1], 3, Some(4))
    );
    assert_eq!(
        annotate(example[2]),
        AnnotatedVial::new(example[2], 4, Some(1))
    );
    assert_eq!(
        annotate(example[3]),
        AnnotatedVial::new(example[3], 2, Some(2))
    );
    assert_eq!(
        annotate(example[4]),
        AnnotatedVial::new(example[4], 1, Some(3))
    );
    assert_eq!(
        annotate(example[5]),
        AnnotatedVial::new(example[5], 0, None)
    );
}
