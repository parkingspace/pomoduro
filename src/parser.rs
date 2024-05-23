use std::{num::ParseIntError, time::Duration};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ParseTimeError {
    #[error("Time must be an integer")]
    ParseIntError(#[from] ParseIntError),
}

pub fn parse_duration(input: &str) -> Result<Duration, ParseTimeError> {
    let mut total_duration = Duration::ZERO;

    do_parse(input, 'h')
        .and_then(|(duration, remaining)| {
            total_duration += duration;
            do_parse(remaining, 'm')
        })
        .and_then(|(duration, remaining)| {
            total_duration += duration;
            do_parse(remaining, 's')
        })
        .map(|(duration, _)| {
            total_duration += duration;
            total_duration
        })
}

fn do_parse(input: &str, unit: char) -> Result<(Duration, &str), ParseTimeError> {
    match input.split_once(unit) {
        Some((time, input)) => {
            let time = time.parse::<u64>()?;
            let duration = match unit {
                'h' => Duration::from_secs(time * 3600),
                'm' => Duration::from_secs(time * 60),
                's' => Duration::from_secs(time),
                _ => unreachable!(),
            };

            Ok((duration, input))
        }
        None => Ok((Duration::ZERO, input)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_times() {
        assert_eq!(parse_duration("30m").unwrap(), Duration::from_secs(30 * 60));
        assert_eq!(parse_duration("1m").unwrap(), Duration::from_secs(60));
        assert_eq!(parse_duration("0m").unwrap(), Duration::ZERO);

        assert!(matches!(
            parse_duration(".m"),
            Err(ParseTimeError::ParseIntError(_))
        ));
    }
}
