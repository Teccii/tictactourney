use crate::book::OpeningBook;
use crate::tourney::{TimeControl, Tourney, TourneyConfig};

pub mod board;
pub mod book;
pub mod engine;
pub mod info;
pub mod score;
pub mod tourney;
pub mod types;
pub mod ugi;
pub mod util;

fn main() {
    let config = TourneyConfig {
        dev_name: "Ristinolla Dev".to_string(),
        dev_path: "./ristinolla-dev".to_string(),
        dev_tc: TimeControl::Clock(8000, 80),
        base_name: "Ristinolla Base".to_string(),
        base_path: "./ristinolla-base".to_string(),
        base_tc: TimeControl::Clock(8000, 80),
        win_adj: None,
        draw_adj: None,
        report_rate: 10,
        num_games: 2000,
        play_pairs: true,
    };
    let book = OpeningBook::open("./perft_5.epd");

    Tourney::new(8, config, book).run();
}
