// Tests that #[prompt] with unknown attribute produces a compile error.

use fastmcp::prompt;
use fastmcp::{Content, PromptMessage, Role};

#[prompt(unknown = "value")]
fn my_prompt() -> Vec<PromptMessage> {
    vec![]
}

fn main() {}
