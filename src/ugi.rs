use crate::{board::Board, types::Square};
use std::fmt;

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

#[derive(Debug, Clone)]
pub enum UgiCommand {
    Ugi,
    NewGame,
    IsReady,
    Search(Vec<SearchLimit>),
    Position { board: Board, moves: Vec<Square> },
    SetOption { name: String, value: String },
    Stop,
    Quit,
}

impl fmt::Display for UgiCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use SearchLimit::*;
        use UgiCommand::*;

        let str = match self {
            Ugi => "ugi".to_string(),
            NewGame => "uginewgame".to_string(),
            IsReady => "isready".to_string(),
            Position { board, moves } => {
                let mut cmd = format!("position fen {} moves ", board.to_fen());
                for mv in moves {
                    cmd += &format!("{} ", mv);
                }

                cmd
            }
            Search(limits) => {
                let mut cmd = "go ".to_string();
                for limit in limits {
                    match limit {
                        XTime(t) => cmd += &format!("xtime {t} "),
                        OTime(t) => cmd += &format!("otime {t} "),
                        XInc(i) => cmd += &format!("xinc {i} "),
                        OInc(i) => cmd += &format!("oinc {i} "),
                        MoveTime(t) => cmd += &format!("movetime {t} "),
                        Nodes(n) => cmd += &format!("nodes {n} "),
                        Depth(d) => cmd += &format!("depth {d} "),
                    }
                }

                cmd
            }
            SetOption { name, value } => format!("setoption name {name} value {value}"),
            Stop => "stop".to_string(),
            Quit => "quit".to_string(),
        };
        write!(f, "{str}")
    }
}
