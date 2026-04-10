use {
    crate::utils::comp_time_arith::{IsSum, SixEqualsFourPlusTwo},
    rand::{Rng, rngs::StdRng, seq::SliceRandom},
    std::marker::PhantomData,
};

pub type Puzzle = GenericPuzzle<5, 6, 4, 2, SixEqualsFourPlusTwo, StdRng>;

/// C - number of different colours
/// N - height of vials
/// K - number of empty vials in starting state
/// Implies total number of vials = C + K
#[derive(Debug)]
pub struct GenericPuzzle<const N: usize, const T: usize, const C: usize, const K: usize, P, R>
where
    P: IsSum<C, K, T>,
{
    inner: [Vial<N>; T],
    original: [Vial<N>; T],
    steps: usize,
    rng: R,
    proof: PhantomData<P>,
}

impl<const N: usize, const T: usize, const C: usize, const K: usize, P, R>
    GenericPuzzle<N, T, C, K, P, R>
where
    P: IsSum<C, K, T>,
    R: Rng,
{
    pub fn new(rng: R) -> Self {
        let inner = [Vial::new([0; N]); T];
        let mut this = Self {
            original: inner,
            inner,
            steps: 0,
            rng,
            proof: PhantomData,
        };

        this.regenerate();

        this
    }

    pub fn regenerate(&mut self) {
        shuffle_state::<N, T, C, R>(&mut self.inner, &mut self.rng);

        if let Some(steps) = crate::solver::solve::<N, T, C>(self.inner) {
            self.steps = steps
        } else {
            self.regenerate();
        }

        self.original = self.inner;
    }

    pub fn get(&self, i: usize) -> &Vial<N> {
        &self.inner[i]
    }

    pub fn min_moves(&self) -> usize {
        self.steps
    }

    pub fn is_solved(&self) -> bool {
        self.inner.iter().all(|vial| {
            let expected = &vial.inner[0];
            vial.inner.iter().all(|c| c == expected)
        })
    }

    pub fn reset(&mut self) {
        self.inner = self.original;
    }

    pub fn pour(&mut self, src: usize, dst: usize) -> bool {
        if src == dst {
            return false;
        }

        let vials = &mut self.inner;
        let Ok([vial1, vial2]) = vials.get_disjoint_mut([src, dst]) else {
            return false;
        };

        pour(vial1, vial2)
    }
}

/// Note: does not guarantee the result is solvable!
pub fn shuffle_state<const N: usize, const T: usize, const C: usize, R>(
    state: &mut [Vial<N>; T],
    rng: &mut R,
) where
    R: Rng,
{
    // Set up buffer with all the colours and shuffle it
    let mut buf = Vec::with_capacity(C * N);
    for c in 1..=C {
        for _ in 0..N {
            buf.push(c as u8);
        }
    }
    buf.shuffle(rng);

    // Fill first C vials
    let iter = state.iter_mut().enumerate().take(C);
    for (i, v) in iter {
        let s = i * N;
        let e = s + N;
        v.inner.copy_from_slice(&buf[s..e]);
    }

    // Last K vials are empty
    for v in state.iter_mut().skip(C) {
        for cell in v.inner.iter_mut() {
            *cell = 0;
        }
    }
}

pub fn pour<const N: usize>(src: &mut Vial<N>, dst: &mut Vial<N>) -> bool {
    let Some(mut from) = src.inner.iter().position(|c| c > &0) else {
        // Pouring is impossible if src is empty
        return false;
    };

    let Some(mut to) = dst
        .inner
        .iter()
        .enumerate()
        .rev()
        .find_map(|(i, c)| if c == &0 { Some(i) } else { None })
    else {
        // Pouring is impossible if dst is full
        return false;
    };

    let c = src.inner[from];
    while from < N && src.inner[from] == c {
        src.inner[from] = 0;
        dst.inner[to] = c;
        if to == 0 {
            break;
        }
        to -= 1;
        from += 1;
    }

    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Vial<const N: usize> {
    inner: [u8; N],
}

impl<const N: usize> Vial<N> {
    pub const fn new(inner: [u8; N]) -> Self {
        Self { inner }
    }

    pub fn get(&self, i: usize) -> u8 {
        self.inner[i]
    }

    pub fn iter(&self) -> impl Iterator<Item = &u8> {
        self.inner.iter()
    }
}
