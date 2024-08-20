use crate::parser::parse_duration;
use clap::{Parser, Subcommand};
use std::time::Duration;

#[derive(Parser)]
#[command(name = "Pomodoro Timer")]
#[command(version = "0.1")]
#[command(about = "Pomodoro Timer", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Start a timer", visible_alias = "t")]
    Timer {
        #[arg(value_parser = parse_duration, short, long)]
        duration: Duration,
        #[arg(short, long)]
        name: Option<String>,
    },

    #[command(about = "Start a pomodoro session", visible_alias = "p")]
    Pomodoro {
        #[arg(short, long)]
        sessions: Option<usize>,
        #[arg(short, long="focus", value_parser = parse_duration)]
        focus_duration: Option<Duration>,
        #[arg(short, long="break", value_parser = parse_duration)]
        break_duration: Option<Duration>,
        #[arg(short, long="long", value_parser = parse_duration)]
        long_break_duration: Option<Duration>,
    },

    #[command(about = "Host a shared pomodoro session", visible_alias = "h")]
    Host {
        #[arg(short, long)]
        port: Option<u16>,
    },

    #[command(about = "Join a shared pomodoro session", visible_alias = "j")]
    Join {
        #[arg(short, long)]
        address: Option<String>,
        #[arg(short, long)]
        port: Option<u16>,
    },
}

pub fn parse() -> Cli {
    Cli::parse()
}
