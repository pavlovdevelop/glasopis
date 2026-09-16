// Няма конзолен прозорец при стартиране на Windows в release режим.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    glasopis_lib::run();
}
