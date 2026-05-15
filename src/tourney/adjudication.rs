use crate::score::Score;
use crate::types::Piece;
use enum_map::EnumMap;

#[derive(Debug, Copy, Clone)]
pub struct DrawAdjudication {
    mv_count: usize,
    mv_num: usize,
    score: i32,
}

#[derive(Debug, Copy, Clone)]
pub struct WinAdjudication {
    mv_count: usize,
    score: i32,
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct DrawTracker {
    config: DrawAdjudication,
    moves: usize,
}

impl DrawTracker {
    #[inline]
    pub fn new(config: DrawAdjudication) -> Self {
        DrawTracker { config, moves: 0 }
    }

    #[inline]
    pub fn update(&mut self, ply: usize, score: Score) {
        if ply >= self.config.mv_num && score.abs().0 <= self.config.score {
            self.moves += 1;
        } else {
            self.moves = 0;
        }
    }

    #[inline]
    pub fn adj(&self) -> bool {
        self.moves >= self.config.mv_count
    }
}

#[derive(Debug, Copy, Clone)]
pub struct WinTracker {
    config: WinAdjudication,
    moves: EnumMap<Piece, usize>,
}

impl WinTracker {
    #[inline]
    pub fn new(config: WinAdjudication) -> Self {
        WinTracker {
            config,
            moves: EnumMap::default(),
        }
    }

    #[inline]
    pub fn update(&mut self, stm: Piece, score: Score) {
        if score.abs().0 >= self.config.score {
            self.moves[stm] += 1;
        } else {
            self.moves[stm] = 0;
        }
    }

    #[inline]
    pub fn adj(&self) -> Option<Piece> {
        for &piece in Piece::ALL {
            if self.moves[piece] >= self.config.mv_count {
                return Some(piece);
            }
        }

        None
    }
}
