use crate::types::Piece;
use std::time::Duration;

pub const TIME_MARGIN: u64 = 100;

#[derive(Debug, Clone)]
pub enum SearchLimit {
    XTime(u64),
    OTime(u64),
    XInc(u64),
    OInc(u64),
    MoveTime(u64),
    Nodes(u64),
    Depth(u8),
}

#[derive(Debug, Copy, Clone)]
pub enum TimeControl {
    Clock(u64, u64),
    MoveTime(u64),
    Nodes(u64),
    Depth(u8),
}

impl TimeControl {
    #[inline]
    pub fn update(&mut self, time: Duration) {
        if let TimeControl::Clock(t, i) = self {
            *t = (*t as i64 - time.as_millis() as i64).max(0) as u64 + *i;
        }
    }

    #[inline]
    pub fn limits(&self, stm: Piece) -> Vec<SearchLimit> {
        match self {
            TimeControl::Clock(t, i) => match stm {
                Piece::X => vec![SearchLimit::XTime(*t), SearchLimit::OTime(*i)],
                Piece::O => vec![SearchLimit::OTime(*t), SearchLimit::OInc(*i)],
            },
            TimeControl::MoveTime(t) => vec![SearchLimit::MoveTime(*t)],
            TimeControl::Nodes(n) => vec![SearchLimit::Nodes(*n)],
            TimeControl::Depth(d) => vec![SearchLimit::Depth(*d)],
        }
    }

    #[inline]
    pub fn timeout(&self) -> Duration {
        match self {
            TimeControl::Clock(t, _) => Duration::from_millis(*t + TIME_MARGIN),
            TimeControl::MoveTime(t) => Duration::from_millis(*t + TIME_MARGIN),
            TimeControl::Nodes(_) => Duration::from_hours(1),
            TimeControl::Depth(_) => Duration::from_hours(1),
        }
    }
}
