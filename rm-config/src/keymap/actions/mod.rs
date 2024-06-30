use rm_shared::action::Action;

pub mod general;
pub mod torrents_tab;

pub trait UserAction: Into<Action> {
    fn desc(&self) -> &'static str;
}
