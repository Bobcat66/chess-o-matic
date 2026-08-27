// Copyright (c) Jesse Kane
// You may use, distribute, and modify this software under the terms of
// the license found in the root directory of this project

mod greedeval;

use crate::game::BoardAnal;
pub trait Evaluator {
    fn eval(anal: &BoardAnal) -> i32;
}

pub use greedeval::GreedEval;
