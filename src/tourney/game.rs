use crate::{
    board::{Board, TerminalState},
    score::Score,
    tourney::{DrawAdjudication, DrawTracker, WinAdjudication, WinTracker},
    types::Square,
    ugi::UgiCommand,
};

pub struct Game {
    win_tracker: Option<WinTracker>,
    draw_tracker: Option<DrawTracker>,
    moves: Vec<Square>,
    current: Board,
    startpos: Board,
}

impl Game {
    #[inline]
    pub fn new(
        startpos: Board,
        win_adj: Option<WinAdjudication>,
        draw_adj: Option<DrawAdjudication>,
    ) -> Self {
        Game {
            win_tracker: win_adj.map(WinTracker::new),
            draw_tracker: draw_adj.map(DrawTracker::new),
            moves: Vec::new(),
            current: startpos,
            startpos,
        }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn make_move(&mut self, score: Option<Score>, mv: Square) -> bool {
        if !self.current.is_legal(mv) {
            return false;
        }

        if let Some(score) = score {
            if let Some(mut win_tracker) = self.win_tracker {
                win_tracker.update(self.current.stm(), score);
            }

            if let Some(mut draw_tracker) = self.draw_tracker {
                draw_tracker.update(self.current.ply() as usize, score);
            }
        }

        self.current.make_move(mv);
        self.moves.push(mv);

        true
    }

    #[inline]
    pub fn terminal_state(&self) -> Option<TerminalState> {
        self.current
            .terminal_state()
            .or_else(|| {
                self.draw_tracker
                    .is_some_and(|d| d.adj())
                    .then_some(TerminalState::Draw)
            })
            .or_else(|| {
                self.win_tracker
                    .and_then(|w| w.adj())
                    .map(|p| TerminalState::Victory(!p))
            })
    }

    #[inline]
    pub fn pos_cmd(&self) -> UgiCommand {
        UgiCommand::Position {
            board: self.startpos,
            moves: self.moves.clone(),
        }
    }
}
