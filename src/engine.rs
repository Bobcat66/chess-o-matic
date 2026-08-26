// Copyright (c) Jesse Kane
// You may use, distribute, and modify this software under the terms of
// the license found in the root directory of this project


use crate::game::{Board, BoardAnal ,Color};

mod search;
// Evaluator trait
pub trait Evaluator {
    fn eval(anal: &BoardAnal) -> i32;
}

