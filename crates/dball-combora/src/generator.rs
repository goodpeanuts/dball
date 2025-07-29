use crate::dball::{DBall, DBallBatch};

pub trait RandomGenerator {
    fn generate_batch() -> anyhow::Result<[DBall; 5]>;

    fn evaluate_batch(batch: &DBallBatch) -> f64;
}
