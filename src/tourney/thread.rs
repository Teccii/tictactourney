use crate::{
    board::{Board, TerminalState},
    engine::Engine,
    info::InfoLine,
    tourney::{Game, TourneyConfig},
    types::{Piece, Square},
    ugi::{SearchLimit, UgiCommand},
};
use std::{
    sync::{Arc, atomic::*},
    time::Duration,
};

#[derive(Default)]
pub struct SharedData {
    wins: AtomicUsize,
    draws: AtomicUsize,
    losses: AtomicUsize,
    penta: [AtomicUsize; 5],
    stop: AtomicU32,
}

impl SharedData {
    #[inline]
    pub fn set_stop(&self, stop: bool) {
        self.stop.store(stop as u32, Ordering::Relaxed);
        atomic_wait::wake_all(&self.stop);
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn should_stop(&self) -> bool {
        self.stop.load(Ordering::Relaxed) != 0
    }

    #[inline]
    pub fn penta(&self) -> [usize; 5] {
        std::array::from_fn(|i| self.penta[i].load(Ordering::Relaxed))
    }

    #[inline]
    pub fn num_games(&self) -> usize {
        self.wins() + self.draws() + self.losses()
    }

    #[inline]
    pub fn num_pairs(&self) -> usize {
        self.penta().iter().sum()
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn wins(&self) -> usize {
        self.wins.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn draws(&self) -> usize {
        self.draws.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn losses(&self) -> usize {
        self.losses.load(Ordering::Relaxed)
    }
}

#[inline]
pub fn thread_loop(config: TourneyConfig, shared: Arc<SharedData>) {
    let mut dev = Engine::new(&config.dev_path);
    let mut base = Engine::new(&config.base_path);

    dev.ping();
    base.ping();

    while shared.num_games() < config.num_games {
        use TerminalState::*;
        if shared.should_stop() {
            return;
        }

        let record_result = |result: TerminalState, dev: Piece| {
            match result {
                Victory(p) if p == dev => shared.wins.fetch_add(1, Ordering::Relaxed),
                Victory(_) => shared.losses.fetch_add(1, Ordering::Relaxed),
                Draw => shared.draws.fetch_add(1, Ordering::Relaxed),
            };
        };

        let opening = Board::default();
        let game1 = play(
            &mut Game::new(opening, config.win_adj, config.draw_adj),
            &mut dev,
            &mut base,
        );

        if !config.play_pairs {
            record_result(game1, opening.stm());
            continue;
        }

        let game2 = play(
            &mut Game::new(opening, config.win_adj, config.draw_adj),
            &mut base,
            &mut dev,
        );
        record_result(game1, opening.stm());
        record_result(game2, !opening.stm());

        let game1_score = match game1 {
            Victory(p) if p == opening.stm() => 2,
            Victory(_) => 0,
            Draw => 1,
        };
        let game2_score = match game2 {
            Victory(p) if p != opening.stm() => 2,
            Victory(_) => 0,
            Draw => 1,
        };

        shared.penta[game1_score + game2_score].fetch_add(1, Ordering::Relaxed);
    }
}

#[inline]
fn play(game: &mut Game, stm: &mut Engine, ntm: &mut Engine) -> TerminalState {
    stm.send(UgiCommand::NewGame);
    ntm.send(UgiCommand::NewGame);

    loop {
        if let Some(terminal_state) = game.terminal_state() {
            return terminal_state;
        }

        let (info_line, best_move) = search(game, stm);
        if !game.make_move(info_line.score, best_move) {
            panic!("Illegal Move");
        }

        if let Some(terminal_state) = game.terminal_state() {
            return terminal_state;
        }

        let (info_line, best_move) = search(game, ntm);
        if !game.make_move(info_line.score, best_move) {
            panic!("Illegal Move");
        }
    }
}

#[inline]
fn search(game: &Game, engine: &mut Engine) -> (InfoLine, Square) {
    engine.ping();
    engine.send(game.pos_cmd());

    engine.ping();
    engine.send(UgiCommand::Search(vec![SearchLimit::MoveTime(100)]));

    let (info_lines, best_move) = engine.read_until(Duration::from_millis(200), |line| {
        line.starts_with("bestmove ")
    });
    let last_info = InfoLine::parse(info_lines.last().unwrap()).unwrap();
    let best_move = best_move
        .split_ascii_whitespace()
        .last()
        .unwrap()
        .parse::<Square>()
        .unwrap();

    (last_info, best_move)
}
