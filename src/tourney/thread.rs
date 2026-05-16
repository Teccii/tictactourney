use crate::{
    board::TerminalState,
    book::OpeningBook,
    engine::Engine,
    info::InfoLine,
    tourney::{Game, TimeControl, TourneyConfig},
    types::Square,
    ugi::UgiCommand,
};
use std::{
    sync::{Arc, atomic::*},
    time::{Duration, Instant},
};

pub struct SharedData {
    book: OpeningBook,
    wins: AtomicUsize,
    draws: AtomicUsize,
    losses: AtomicUsize,
    penta: [AtomicUsize; 5],
    game_id: AtomicUsize,
    stop: AtomicU32,
}

impl SharedData {
    #[inline]
    pub fn new(book: OpeningBook) -> Self {
        SharedData {
            book,
            wins: AtomicUsize::new(0),
            draws: AtomicUsize::new(0),
            losses: AtomicUsize::new(0),
            penta: std::array::from_fn(|_| AtomicUsize::new(0)),
            game_id: AtomicUsize::new(0),
            stop: AtomicU32::new(0),
        }
    }

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
    pub fn next_game_id(&self) -> usize {
        self.game_id.fetch_add(1, Ordering::Relaxed)
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
pub fn thread_loop(id: usize, config: TourneyConfig, shared: Arc<SharedData>) {
    let mut dev = Engine::new(&config.dev_path);
    let mut base = Engine::new(&config.base_path);

    dev.send(UgiCommand::Ugi);
    dev.wait_for(Duration::from_millis(1000), |out| out.starts_with("ugiok"));
    base.send(UgiCommand::Ugi);
    base.wait_for(Duration::from_millis(1000), |out| out.starts_with("ugiok"));

    dev.ping();
    base.ping();

    let mut rng = rand::rng();
    while shared.num_games() < config.num_games {
        use TerminalState::*;
        if shared.should_stop() {
            return;
        }

        let opening = shared.book.next(&mut rng);
        let game1_id = shared.next_game_id();
        println!(
            "Thread {id}: Starting Match #{game1_id} between {} and {}",
            config.dev_name, config.base_name
        );
        let game1 = play(
            &mut Game::new(opening, config.win_adj, config.draw_adj),
            config.dev_tc.clone(),
            config.base_tc.clone(),
            &mut dev,
            &mut base,
        );

        let (game1_result, game1_score) = match game1 {
            Victory(p) if p == opening.stm() => (format!("{} won", config.dev_name), 2),
            Victory(_) => (format!("{} won", config.base_name), 0),
            Draw => ("draw".to_string(), 1)
        };

        println!(
            "Thread {id}: Finished Match #{game1_id} between {} and {} ({})",
            config.dev_name, config.base_name, game1_result
        );
        if !config.play_pairs {
            continue;
        }

        let game2_id = shared.next_game_id();
        println!(
            "Thread {id}: Starting Game #{game2_id} between {} and {}",
            config.base_name, config.dev_name
        );
        let game2 = play(
            &mut Game::new(opening, config.win_adj, config.draw_adj),
            config.base_tc.clone(),
            config.dev_tc.clone(),
            &mut base,
            &mut dev,
        );

        let (game2_result, game2_score) = match game2 {
            Victory(p) if p != opening.stm() => (format!("{} won", config.dev_name), 2),
            Victory(_) => (format!("{} won", config.base_name), 0),
            Draw => ("draw".to_string(), 1),
        };
        println!(
            "Thread {id}: Finished Match #{game2_id} between {} and {} ({})",
            config.base_name, config.dev_name, game2_result
        );
        shared.penta[game1_score + game2_score].fetch_add(1, Ordering::Relaxed);

        match game1 {
            Victory(p) if p == opening.stm() => shared.wins.fetch_add(1, Ordering::Relaxed),
            Victory(_) => shared.losses.fetch_add(1, Ordering::Relaxed),
            Draw => shared.draws.fetch_add(1, Ordering::Relaxed),
        };

        match game2 {
            Victory(p) if p != opening.stm() => shared.wins.fetch_add(1, Ordering::Relaxed),
            Victory(_) => shared.losses.fetch_add(1, Ordering::Relaxed),
            Draw => shared.draws.fetch_add(1, Ordering::Relaxed),
        };
    }
}

#[inline]
fn play(
    game: &mut Game,
    mut stm_tc: TimeControl,
    mut ntm_tc: TimeControl,
    stm: &mut Engine,
    ntm: &mut Engine,
) -> TerminalState {
    stm.send(UgiCommand::NewGame);
    ntm.send(UgiCommand::NewGame);

    loop {
        if let Some(terminal_state) = game.terminal_state() {
            return terminal_state;
        }

        let (info_line, best_move) = search(game, stm, &mut stm_tc, &ntm_tc);
        if !game.make_move(info_line.score, best_move) {
            panic!("Illegal Move");
        }

        if let Some(terminal_state) = game.terminal_state() {
            return terminal_state;
        }

        let (info_line, best_move) = search(game, ntm, &mut ntm_tc, &stm_tc);
        if !game.make_move(info_line.score, best_move) {
            panic!("Illegal Move");
        }
    }
}

#[inline]
fn search(
    game: &Game,
    stm: &mut Engine,
    stm_tc: &mut TimeControl,
    ntm_tc: &TimeControl,
) -> (InfoLine, Square) {
    let mut limits = stm_tc.limits(game.board().stm());
    if let TimeControl::Clock(_, _) = ntm_tc {
        limits.extend(ntm_tc.limits(!game.board().stm()));
    }

    stm.ping();
    stm.send(game.pos_cmd());

    stm.ping();
    stm.send(UgiCommand::Search(limits));

    let instant = Instant::now();
    let (info_lines, best_move) = stm
        .read_until(stm_tc.timeout(), |line| line.starts_with("bestmove "))
        .unwrap();
    stm_tc.update(instant.elapsed());

    let last_info = InfoLine::parse(info_lines.last().unwrap()).unwrap();
    let best_move = best_move
        .split_ascii_whitespace()
        .last()
        .unwrap()
        .parse::<Square>()
        .unwrap();

    (last_info, best_move)
}
