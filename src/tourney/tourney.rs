use crate::tourney::{DrawAdjudication, SharedData, WinAdjudication, thread_loop};
use std::{sync::Arc, thread::JoinHandle};

/*----------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub struct TourneyConfig {
    pub dev_name: String,
    pub dev_path: String,
    pub base_name: String,
    pub base_path: String,
    pub win_adj: Option<WinAdjudication>,
    pub draw_adj: Option<DrawAdjudication>,
    pub num_games: usize,
    pub play_pairs: bool,
}

/*----------------------------------------------------------------*/

pub struct Tourney {
    pub shared: Arc<SharedData>,
    search_threads: Vec<JoinHandle<()>>,
}

impl Tourney {
    #[inline]
    pub fn new(num_threads: usize, config: TourneyConfig) -> Self {
        let shared = Arc::new(SharedData::default());
        let search_threads = (0..num_threads)
            .map(|_| {
                std::thread::spawn({
                    let config = config.clone();
                    let shared = shared.clone();

                    move || {
                        if std::panic::catch_unwind(move || thread_loop(config, shared)).is_err() {
                            std::process::exit(1);
                        }
                    }
                })
            })
            .collect();

        Tourney {
            shared,
            search_threads,
        }
    }

    #[inline]
    pub fn quit(&mut self) {
        self.shared.set_stop(true);
        self.search_threads
            .drain(..)
            .for_each(|t| t.join().unwrap());
    }
}
