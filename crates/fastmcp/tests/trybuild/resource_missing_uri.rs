// Tests that #[resource] without uri attribute produces a compile error.

use fastmcp::resource;

#[resource]
fn my_resource() -> String {
    "data".to_string()
}

fn main() {}
