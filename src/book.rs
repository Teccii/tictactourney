use crate::board::Board;
use rand::{Rng, prelude::IndexedRandom};
use std::{
    fs::OpenOptions,
    io::{BufRead, BufReader},
};

pub struct OpeningBook(Vec<Board>);

impl OpeningBook {
    #[inline]
    pub fn open(path: &str) -> Self {
        let file = OpenOptions::new().read(true).open(path).unwrap();

        OpeningBook(
            BufReader::new(file)
                .lines()
                .map(|s| Board::from_fen(&s.unwrap()).unwrap())
                .collect(),
        )
    }

    #[inline]
    pub fn next<R: Rng + ?Sized>(&self, rng: &mut R) -> Board {
        self.0.choose(rng).unwrap().clone()
    }
}
