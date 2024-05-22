use std::{num::ParseIntError, time::Duration};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum MyError {
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
}

pub fn parse_duration(input: &str) -> Result<Duration, MyError> {
    let (mins, _) = match input.split_once('m') {
        Some((mins, input)) => {
            let mins = mins.parse::<u64>()?;
            (Duration::from_secs(mins * 60), input)
        }
        None => (Duration::ZERO, input),
    };

    Ok(mins)
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
            parse_duration("am"),
            Err(MyError::ParseIntError(_))
        ));
    }
}
