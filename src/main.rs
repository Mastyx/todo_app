#![allow(warnings)]

// direttiva per includere il modulo task
mod app;
mod gui;
mod task;

use crate::app::Task_app;
use std::io::Result;
// ---------------------------------------------------------------
fn main() -> Result<()> {
    gui::run()
}
