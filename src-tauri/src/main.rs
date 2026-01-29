// Prevents additional console window on Windows in release, DO NOT REMOVE!!
// You can directly open the program in the terminal to view the logs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    nanokvm_testing_v2_lib::run()
}
