use std::{num::ParseIntError, time::Duration};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ParseTimeError {
    #[error("Time must be an integer")]
    IntError(#[from] ParseIntError),

    #[error("Invalid unit format: {0}")]
    InvalidUnit(String),

    #[error("Invalid character in time format: {0}")]
    InvalidChar(String),
}

pub fn parse_duration(input: &str) -> Result<Duration, ParseTimeError> {
    let mut total_duration = Duration::ZERO;

    do_parse(input.trim(), 'h')
        .and_then(|(duration, remaining)| {
            total_duration += duration;
            do_parse(remaining, 'm')
        })
        .and_then(|(duration, remaining)| {
            total_duration += duration;
            do_parse(remaining, 's')
        })
        .and_then(|(duration, remaining)| {
            total_duration += duration;
            if remaining.is_empty() {
                Ok(total_duration)
            } else {
                Err(ParseTimeError::InvalidUnit(remaining.to_string()))
            }
        })
}

fn do_parse(input: &str, unit: char) -> Result<(Duration, &str), ParseTimeError> {
    match input.split_once(unit) {
        Some((time, input)) => match time.parse::<u64>() {
            Ok(time) => {
                let duration = match unit {
                    'h' => Duration::from_secs(time * 3600),
                    'm' => Duration::from_secs(time * 60),
                    's' => Duration::from_secs(time),
                    _ => unreachable!(),
                };

                Ok((duration, input))
            }
            Err(e) => {
                if time.chars().any(|c| c.is_alphabetic()) {
                    Err(ParseTimeError::InvalidChar(time.to_string()))
                } else {
                    Err(ParseTimeError::IntError(e))
                }
            }
        },
        None => Ok((Duration::ZERO, input)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hours() {
        assert_eq!(
            parse_duration("30h").unwrap(),
            Duration::from_secs(30 * 3600)
        );
        assert_eq!(parse_duration("1h").unwrap(), Duration::from_secs(3600));
        assert_eq!(parse_duration("0h").unwrap(), Duration::ZERO);
    }

    #[test]
    fn test_minutes() {
        assert_eq!(parse_duration("30m").unwrap(), Duration::from_secs(30 * 60));
        assert_eq!(parse_duration("1m").unwrap(), Duration::from_secs(60));
        assert_eq!(parse_duration("0m").unwrap(), Duration::ZERO);
    }

    #[test]
    fn test_seconds() {
        assert_eq!(parse_duration("30s").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_duration("1s").unwrap(), Duration::from_secs(1));
        assert_eq!(parse_duration("0s").unwrap(), Duration::ZERO);
    }

    #[test]
    fn test_combined() {
        assert_eq!(
            parse_duration("1h30m").unwrap(),
            Duration::from_secs(90 * 60)
        );
        assert_eq!(
            parse_duration("5h20m20s").unwrap(),
            Duration::from_secs(5 * 3600 + 20 * 60 + 20)
        );
    }

    #[test]
    fn test_invalid() {
        assert!(matches!(
            parse_duration("1ma"),
            Err(ParseTimeError::InvalidUnit(_))
        ));
        assert!(matches!(
            parse_duration("am"),
            Err(ParseTimeError::InvalidChar(_))
        ));
        assert!(matches!(
            parse_duration("0.5m"),
            Err(ParseTimeError::IntError(_))
        ));
    }
}
