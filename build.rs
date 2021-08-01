fn main() {
	prost_build::compile_protos(&["src/data/message.proto"], &["src/"]).unwrap();

	// ***** NOTE ******
	// Uncomment this so as to copy the new file

	//use std::{
	// 	env, fs,
	// 	path::{Path, PathBuf},
	// };
	// let dst = PathBuf::from(env::var_os("OUT_DIR").unwrap());
	// let tmp_generated_message = dst.join("messages.rs");
	// let output_file = PathBuf::from("src/generated/messages.rs");
	// fs::copy(tmp_generated_message, output_file).unwrap();
}
