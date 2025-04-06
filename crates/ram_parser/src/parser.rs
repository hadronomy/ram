//! See [`Parser`].

use std::cell::Cell;

use crate::{Event, Input};

pub struct Parser<'t> {
    inp: &'t Input,
    pos: usize,
    events: Vec<Event>,
    steps: Cell<u32>,
}
