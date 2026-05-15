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
    pub fn parse(line: &str) -> anyhow::Result<InfoLine> {
        let mut reader = line.split_ascii_whitespace();

        if reader.next().is_none_or(|l| l != "info") {
            return Err(anyhow::anyhow!("Missing `info` token in info line"));
        }

        #[inline]
        fn parse_int<T: FromStr<Err = ParseIntError>>(
            reader: &mut SplitAsciiWhitespace,
            token: &str,
        ) -> anyhow::Result<T> {
            Ok(reader
                .next()
                .ok_or_else(|| anyhow::anyhow!("Missing integer value for `{token}` in info line"))?
                .parse::<T>()?)
        }

        let mut info_line = InfoLine::default();
        while let Some(token) = reader.next() {
            match token {
                "depth" => info_line.depth = Some(parse_int::<u8>(&mut reader, token)?),
                "seldepth" => info_line.seldepth = Some(parse_int::<u8>(&mut reader, token)?),
                "score" => match reader.next() {
                    Some("cp") => {
                        info_line.score = Some(Score(parse_int::<i32>(&mut reader, token)?))
                    }
                    Some("mate") => {
                        info_line.score = {
                            let ply = parse_int::<i32>(&mut reader, token)?;

                            if ply > 0 {
                                Some(Score::mate(ply as u8))
                            } else {
                                Some(Score::mate((-ply) as u8))
                            }
                        }
                    }
                    _ => {}
                },
                "time" => info_line.time = Some(parse_int::<u64>(&mut reader, token)?),
                "nodes" => info_line.nodes = Some(parse_int::<u64>(&mut reader, token)?),
                _ => {}
            }
        }

        Ok(info_line)
    }
}
