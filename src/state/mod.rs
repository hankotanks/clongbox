use std::borrow;

pub mod focus;

#[derive(Default)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct State;

impl State {
    pub fn parse_from_str<'a, I>(_content: I) -> anyhow::Result<Self> 
        where I: Into<borrow::Cow<'a, str>> + Clone {

        anyhow::bail!("Unimplemented!");
    }
}