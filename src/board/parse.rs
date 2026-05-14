use crate::{
    board::{Board, TerminalState},
    types::{File, Piece, Rank, Square},
};

impl Board {
    #[inline]
    pub fn from_fen(fen: &str) -> Option<Board> {
        let mut parts = fen.trim().split_ascii_whitespace();
        let pieces = parts.next()?;
        let stm = parts.next()?;
        let ply = parts.next()?;
        let prev_move = parts.next()?;

        if parts.next().is_some() {
            return None;
        }

        if stm.len() != 1 {
            return None;
        }

        let mut board = Board::default();
        board.ply = ply.parse::<u8>().ok()?.max(0);
        board.prev_mv = prev_move.parse::<Square>().ok();
        board.stm = stm.chars().next().and_then(|c| Piece::try_from(c.to_ascii_lowercase()).ok())?;

        for (rank, row) in pieces.rsplit('/').enumerate() {
            let rank = Rank::try_index(rank)?;
            let mut file = 0;

            for p in row.chars() {
                if let Some(empty) = p.to_digit(10) {
                    file += empty as usize;
                } else {
                    let sq = Square::new(File::try_index(file)?, rank);
                    let piece = Piece::try_from(p.to_ascii_lowercase()).ok()?;
                    let indices = sq.indices();

                    board.small[indices.0].set(piece, indices.1);
                    file += 1;
                }
            }

            if file != File::COUNT {
                return None;
            }
        }

        for i in 0..9 {
            if let Some(TerminalState::Victory(piece)) = board.small[i].terminal_state() {
                board.large.set(piece, i);
            }
        }

        Some(board)
    }

    #[inline]
    pub fn to_fen(&self) -> String {
        let mut fen = String::new();
        for &rank in Rank::ALL.iter().rev() {
            let mut empty = 0;
            for &file in File::ALL.iter() {
                let sq = Square::new(file, rank);

                if let Some(piece) = self.piece_on(sq) {
                    if empty > 0 {
                        fen += &format!("{empty}");
                        empty = 0;
                    }

                    fen += &format!("{}", piece);
                } else {
                    empty += 1;
                }
            }

            if empty > 0 {
                fen += &format!("{empty}");
            }

            if rank > Rank::First {
                fen += "/";
            }
        }

        fen + &format!(" {}", self.stm)
    }
}
