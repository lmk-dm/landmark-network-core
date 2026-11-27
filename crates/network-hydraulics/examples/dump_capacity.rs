//! Emit the two-pad capacity contract as JSON (Field Network consumer path).
//!
//! ```bash
//! cargo run -p network-hydraulics --example dump_capacity
//! ```

use network_hydraulics::dump_two_pad_capacity;

fn main() {
    let dump = dump_two_pad_capacity(48.0).expect("two-pad capacity dump");
    println!("{}", serde_json::to_string_pretty(&dump).expect("json"));
}
