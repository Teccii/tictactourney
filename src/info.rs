use crate::score::Score;
use std::str::SplitAsciiWhitespace;
use std::{num::ParseIntError, str::FromStr};

#[derive(Debug, Copy, Clone, Default)]
pub struct InfoLine {
    pub depth: Option<u8>,
    pub seldepth: Option<u8>,
    pub score: Option<Score>,
    pub time: Option<u64>,
    pub nodes: Option<u64>,
}

impl InfoLine {
    pub fn parse(line: &str) -> Option<InfoLine> {
        let mut reader = line.split_ascii_whitespace();

        if reader.next().is_none_or(|l| l != "info") {
            return None;
        }

        #[inline]
        fn parse_int<T: FromStr<Err = ParseIntError>>(
            reader: &mut SplitAsciiWhitespace,
        ) -> Option<T> {
            reader.next().unwrap().parse::<T>().ok()
        }

        let mut info_line = InfoLine::default();
        while let Some(token) = reader.next() {
            match token {
                "depth" => info_line.depth = parse_int::<u8>(&mut reader),
                "seldepth" => info_line.seldepth = parse_int::<u8>(&mut reader),
                "score" => match reader.next() {
                    Some("cp") => {
                        info_line.score = parse_int::<i32>(&mut reader).map(Score);
                    }
                    Some("mate") => {
                        info_line.score = parse_int::<i32>(&mut reader).map(|ply| {
                            if ply > 0 {
                                Score::mate(ply as u8)
                            } else {
                                Score::mated((-ply) as u8)
                            }
                        });
                    }
                    _ => {}
                },
                "time" => info_line.time = parse_int::<u64>(&mut reader),
                "nodes" => info_line.nodes = parse_int::<u64>(&mut reader),
                _ => {}
            }
        }

        Some(info_line)
    }
}
