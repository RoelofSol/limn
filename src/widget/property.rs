use std::collections::BTreeSet;

use widget::{EventHandler, EventArgs};

#[derive(Debug)]
pub enum PropChange {
    Add(Property),
    Remove(Property),
}

#[derive(Hash, PartialEq, Eq, Clone, PartialOrd, Ord, Debug)]
pub enum Property {
    Hover,
    Activated,
    Selected,
    Pressed,
    Inactive,
}
pub type PropSet = BTreeSet<Property>;

pub mod states {
    use super::{Property, PropSet};
    lazy_static! {
        pub static ref STATE_DEFAULT: PropSet = btreeset!{};
        pub static ref STATE_HOVER: PropSet = btreeset!{Property::Hover};
        pub static ref STATE_PRESSED: PropSet = btreeset!{Property::Pressed};
        pub static ref STATE_ACTIVATED: PropSet = btreeset!{Property::Activated};
        pub static ref STATE_ACTIVATED_PRESSED: PropSet = btreeset!{Property::Activated, Property::Pressed};
        pub static ref STATE_SELECTED: PropSet = btreeset!{Property::Selected};
        pub static ref STATE_INACTIVE: PropSet = btreeset!{Property::Inactive};
    }
}

pub struct PropChangeHandler;
impl EventHandler<PropChange> for PropChangeHandler {
    fn handle(&mut self, event: &PropChange, mut args: EventArgs) {
        match *event {
            PropChange::Add(ref property) => args.widget.props.insert(property.clone()),
            PropChange::Remove(ref property) => args.widget.props.remove(&property),
        };
        args.widget.apply_style();
    }
}
