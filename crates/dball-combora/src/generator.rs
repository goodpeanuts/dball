use crate::checker::DBallChecker;
use crate::dball::{DBall, DBallBatch, DBallError};
use std::collections::HashSet;

pub enum Generator {
    BlueMorn,
}

impl AsRef<Self> for Generator {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl Generator {
    pub fn create_generator(generator: impl AsRef<Self>) -> Box<dyn RandomGenerator> {
        match generator.as_ref() {
            Self::BlueMorn => Box::new(bluemorn::BlueMorn),
        }
    }
}

pub trait RandomGenerator {
    fn generate_batch(&self) -> anyhow::Result<[DBall; 5]>;

    fn evaluate_batch(&self, batch: &DBallBatch) -> f64;
}

pub mod bluemorn;
