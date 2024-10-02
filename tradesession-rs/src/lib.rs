pub mod jcswitch;
mod tradesession;

use anyhow::Result;
use std::collections::HashMap;

pub use tradesession::*;

pub fn load_tradessesion() -> Result<HashMap<String, TradeSession>> {
    unimplemented!()
}
