use crate::book::OpeningBook;
use crate::tourney::{DrawAdjudication, SharedData, TimeControl, WinAdjudication, thread_loop};
use std::{sync::Arc, thread::JoinHandle};
/*----------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub struct TourneyConfig {
    pub dev_name: String,
    pub dev_path: String,
    pub dev_tc: TimeControl,
    pub base_name: String,
    pub base_path: String,
    pub base_tc: TimeControl,
    pub win_adj: Option<WinAdjudication>,
    pub draw_adj: Option<DrawAdjudication>,
    pub report_rate: usize,
    pub num_games: usize,
    pub play_pairs: bool,
}

/*----------------------------------------------------------------*/

pub struct Tourney {
    pub config: TourneyConfig,
    pub shared: Arc<SharedData>,
    search_threads: Vec<JoinHandle<()>>,
}

impl Tourney {
    #[inline]
    pub fn new(num_threads: usize, config: TourneyConfig, book: OpeningBook) -> Self {
        let shared = Arc::new(SharedData::new(book));
        let search_threads = (0..num_threads)
            .map(|i| {
                std::thread::spawn({
                    let config = config.clone();
                    let shared = shared.clone();

                    move || {
                        if std::panic::catch_unwind(move || thread_loop(i, config, shared)).is_err()
                        {
                            std::process::exit(1);
                        }
                    }
                })
            })
            .collect();

        Tourney {
            config,
            shared,
            search_threads,
        }
    }

    #[inline]
    pub fn run(&mut self) {
        let mut counter = 0;
        let mut last_num = 6767;

        while self.shared.num_games() < self.config.num_games {
            let games = self.shared.num_games();

            if games != last_num {
                counter += games - last_num;
                last_num = games;
            }

            if counter >= self.config.report_rate {
                counter = 0;
                let wins = self.shared.wins();
                let draws = self.shared.draws();
                let losses = self.shared.losses();
                let penta = self.shared.penta();

                println!(
                    "{} vs {}\nN: {games} W: {wins} D: {draws} L: {losses}\nPenta: {penta:?}",
                    self.config.dev_name, self.config.base_name,
                );
            }
        }

        self.quit();

        let wins = self.shared.wins();
        let draws = self.shared.draws();
        let losses = self.shared.losses();
        let games = self.shared.num_games();
        let penta = self.shared.penta();

        println!(
            "{} vs {}\nN: {games} W: {wins} D: {draws} L: {losses}\nPenta: {penta:?}",
            self.config.dev_name, self.config.base_name,
        );
    }

    #[inline]
    pub fn quit(&mut self) {
        self.shared.set_stop(true);
        self.search_threads
            .drain(..)
            .for_each(|t| t.join().unwrap());
    }
}
