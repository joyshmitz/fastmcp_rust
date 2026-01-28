// Tests that #[tool] with invalid timeout produces a compile error.

use fastmcp::tool;

#[tool(timeout = "invalid")]
fn my_tool() -> String {
    "ok".to_string()
}

fn main() {}
