// Tests that #[resource] with unknown attribute produces a compile error.

use fastmcp::resource;

#[resource(uri = "test://data", unknown = "value")]
fn my_resource() -> String {
    "data".to_string()
}

fn main() {}
