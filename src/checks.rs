use crate::sample::PrometheusSample;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum CheckType {
    Up,
    Fresh,
}

impl CheckType {
    fn is_satisfied_by(&self, value: f64, context: CheckContext) -> bool {
        match self {
            CheckType::Up => value >= 1.0,
            CheckType::Fresh => context.is_fresh(value),
        }
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Check<'a> {
    pub name: &'a str,
    pub check_type: CheckType,
}

#[derive(Debug, Copy, Clone)]
pub struct CheckContext {
    pub now: f64,
    pub stale_threshold: f64,
}

impl CheckContext {
    pub fn new(stale_threshold: f64) -> Self {
        Self {
            now: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs_f64(),
            stale_threshold,
        }
    }
    fn is_fresh(&self, value: f64) -> bool {
        value >= self.now - self.stale_threshold
    }
}

impl Check<'_> {
    pub fn is_satisfied_by(&self, sample: &PrometheusSample, context: CheckContext) -> bool {
        self.name == sample.name && self.check_type.is_satisfied_by(sample.value, context)
    }
}
