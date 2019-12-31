//! Basic example.
// extern crate aiven_rs;

// use aiven_rs;
use std::{boxed::Box, error::Error, process};

fn example() -> Result<(), Box<dyn Error>> {
	println!("Hello world");
	Ok(())
}

fn main() {
	if let Err(err) = example() {
		println!("error running example: {}", err);
		process::exit(1);
	}
}
