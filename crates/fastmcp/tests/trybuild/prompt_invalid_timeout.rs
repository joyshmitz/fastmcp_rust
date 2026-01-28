// Tests that #[prompt] with invalid timeout produces a compile error.

use fastmcp::prompt;
use fastmcp::{PromptMessage};

#[prompt(timeout = "abc")]
fn my_prompt() -> Vec<PromptMessage> {
    vec![]
}

fn main() {}
