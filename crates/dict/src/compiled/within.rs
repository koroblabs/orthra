//! Every form within a few edits of a word: a Levenshtein automaton the FST is walked with.
//!
//! Bit-parallel, after Wu and Manber: for each number of edits `d` allowed, one integer whose bit
//! `i` says the bytes read so far match the query's first `i` characters with `d` edits or fewer.
//! A character moves every row with a few shifts and ors, so a state is three integers and a
//! partial character, and copying one costs nothing. The first measurement kept a row of a
//! distance table per state instead, an allocation per byte: 2.4 ms a word, p50, on UA-GEC.
//!
//! The FST is UTF-8 bytes and the edits are characters, so a state carries a character's bytes
//! until it is whole.

use fst::Automaton;

/// The most edits a search allows.
pub(crate) const MOST_EDITS: usize = 2;

/// The longest query, in characters: a row is an integer with a bit for each prefix of it.
pub(crate) const LONGEST: usize = 62;

pub(crate) struct Within {
    /// Each distinct character of the query, and the places it is at as bits.
    masks: Vec<(char, u64)>,
    /// The bit of the whole query.
    whole: u64,
    /// The bits of every prefix of it, the whole included.
    prefixes: u64,
    edits: usize,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct State {
    rows: [u64; MOST_EDITS + 1],
    /// The bits of a character not yet read whole, and how many bytes it still needs.
    partial: u32,
    needs: u8,
}

const DEAD: State = State {
    rows: [0; MOST_EDITS + 1],
    partial: 0,
    needs: 0,
};

impl Within {
    /// The forms within `edits` of `query`; `None` for a query longer than [`LONGEST`] or more
    /// edits than [`MOST_EDITS`].
    pub(crate) fn new(query: &str, edits: usize) -> Option<Within> {
        let mut masks: Vec<(char, u64)> = Vec::new();
        let mut length = 0;
        for (at, c) in query.chars().enumerate() {
            if at >= LONGEST {
                return None;
            }
            match masks.iter_mut().find(|(seen, _)| *seen == c) {
                Some((_, mask)) => *mask |= 1 << at,
                None => masks.push((c, 1 << at)),
            }
            length = at + 1;
        }
        (edits <= MOST_EDITS).then(|| Within {
            masks,
            whole: 1 << length,
            prefixes: (1 << (length + 1)) - 1,
            edits,
        })
    }

    fn step(&self, rows: &[u64; MOST_EDITS + 1], c: char) -> [u64; MOST_EDITS + 1] {
        let mask = self
            .masks
            .iter()
            .find(|(seen, _)| *seen == c)
            .map_or(0, |(_, mask)| *mask);
        let mut next = [0; MOST_EDITS + 1];
        next[0] = ((rows[0] & mask) << 1) & self.prefixes;
        for d in 1..=self.edits {
            // Matched; or, with one more edit: the character inserted (the query stays where it
            // was), substituted (both move on), or a query character deleted (the query moves on
            // and the character is still to be read: the row just made, shifted).
            next[d] =
                (((rows[d] & mask) << 1) | rows[d - 1] | (rows[d - 1] << 1) | (next[d - 1] << 1))
                    & self.prefixes;
        }
        next
    }
}

impl Automaton for Within {
    type State = State;

    fn start(&self) -> State {
        let mut rows = [0; MOST_EDITS + 1];
        // Before anything is read, the first `d` query characters are matched by deleting them.
        for (d, row) in rows.iter_mut().enumerate().take(self.edits + 1) {
            *row = ((1 << (d + 1)) - 1) & self.prefixes;
        }
        State {
            rows,
            partial: 0,
            needs: 0,
        }
    }

    fn is_match(&self, state: &State) -> bool {
        state.needs == 0 && state.rows[self.edits] & self.whole != 0
    }

    fn can_match(&self, state: &State) -> bool {
        // The rows grow with the edits allowed, so the last is empty only when all are.
        state.rows[self.edits] != 0
    }

    fn accept(&self, state: &State, byte: u8) -> State {
        if !self.can_match(state) {
            return DEAD;
        }
        let (partial, needs) = if state.needs == 0 {
            match byte {
                0x00..=0x7f => (u32::from(byte), 0),
                0xc0..=0xdf => (u32::from(byte & 0x1f), 1),
                0xe0..=0xef => (u32::from(byte & 0x0f), 2),
                0xf0..=0xf7 => (u32::from(byte & 0x07), 3),
                _ => return DEAD,
            }
        } else if byte & 0xc0 == 0x80 {
            (
                (state.partial << 6) | u32::from(byte & 0x3f),
                state.needs - 1,
            )
        } else {
            return DEAD;
        };
        if needs > 0 {
            return State {
                rows: state.rows,
                partial,
                needs,
            };
        }
        match char::from_u32(partial) {
            Some(c) => State {
                rows: self.step(&state.rows, c),
                partial: 0,
                needs: 0,
            },
            None => DEAD,
        }
    }
}
