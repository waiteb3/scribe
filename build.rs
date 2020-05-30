// Copyright (C) Brandon Waite 2020  - All Rights Reserved
// Unauthorized copying of this file, via any medium, is strictly prohibited
// Proprietary
// Updated by Brandon Waite, May 28 2020

use std::fs;
use std::io::Write;

fn main() -> std::io::Result<()> {
    let _current = fs::read_to_string("src/data.rs")?;
    let init_zsh = fs::read_to_string("src/init.zsh")?;

    let data = fs::File::create("src/data.rs")?;

    // writeln!(& data, "//- START_INLINED:init.zsh")?;
    writeln!(& data, "pub const INIT_ZSH: &str = r###\"")?;
    writeln!(& data, "{}", init_zsh)?;
    writeln!(& data, "\"###;")?;
    // writeln!(& data, "//- END_INLINED:init.zsh")?;

    Ok(())
}
