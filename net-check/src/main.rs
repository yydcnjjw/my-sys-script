use tokio::prelude::*;
use tokio::timer::Interval;
use std::time::Duration;

fn main() {
    tokio::run({
        Interval::new()
    });
}
