use super::DBall;

const RED_BASE: u32 = 0; // Red ball starting bit index
const BLUE_BASE: u32 = 33; // Blue ball starting bit index
const TOTAL_ONES: u32 = 7; // Fixed 7 ones in each one-hot encoding

/// one-hot bits display for dball
/// lower 33 bits for rballs
/// upper 16 bits for bball
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DBallBit {
    bits: u64,
}

impl DBallBit {
    pub fn from_dball(d: &DBall) -> Self {
        let mut bits: u64 = 0;

        // set rballs: 1-33 -> bit (0..=32)
        for &r in &d.rball {
            let idx = RED_BASE + (r as u32 - 1);
            bits |= 1u64 << idx;
        }

        // set bball: 1-16 -> bit (33..=48)
        let b = d.bball;
        let b_idx = BLUE_BASE + (b as u32 - 1);
        bits |= 1u64 << b_idx;

        Self { bits }
    }

    pub fn hamming_distance(&self, other: &Self) -> u32 {
        (self.bits ^ other.bits).count_ones()
    }

    /// Euclidean distance: ||x - y||^2 = Hamming(x, y)
    pub fn euclidean_distance(&self, other: &Self) -> f64 {
        (self.hamming_distance(other) as f64).sqrt()
    }

    /// Cosine similarity: `shared_ones` / `TOTAL_ONES`
    pub fn cosine_similarity(&self, other: &Self) -> f64 {
        let shared_ones = (self.bits & other.bits).count_ones();
        shared_ones as f64 / TOTAL_ONES as f64
    }

    /// Cosine distance: 1 - Cosine similarity
    pub fn cosine_distance(&self, other: &Self) -> f64 {
        1.0 - self.cosine_similarity(other)
    }

    /// shared ones: 1-7
    pub fn shared_ones(&self, other: &Self) -> u32 {
        (self.bits & other.bits).count_ones()
    }

    /// expose bits for serialization/debugging
    pub fn bits(&self) -> u64 {
        self.bits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distances() {
        let a = DBall {
            rball: [1, 5, 8, 12, 25, 30],
            bball: 6,
            magnification: 1,
        };
        let b = DBall {
            rball: [1, 5, 8, 12, 25, 30],
            bball: 6,
            magnification: 1,
        };
        let aa = DBallBit::from_dball(&a);
        let bb = DBallBit::from_dball(&b);

        assert_eq!(aa.hamming_distance(&bb), 0);
        assert_eq!(aa.euclidean_distance(&bb), 0.0);
        assert!((aa.cosine_distance(&bb) - 0.0).abs() < 1e-12);

        // only blue ball different
        let c = DBall {
            rball: [1, 5, 8, 12, 25, 30],
            bball: 7,
            magnification: 1,
        };
        let cc = DBallBit::from_dball(&c);
        assert_eq!(aa.shared_ones(&cc), 6);
        assert_eq!(aa.hamming_distance(&cc), 2);
        assert_eq!(aa.euclidean_distance(&cc), (2f64).sqrt());
        assert!((aa.cosine_similarity(&cc) - (6.0 / 7.0)).abs() < 1e-12);

        // 4 red balls same, blue ball same → shared 5 (4 red + 1 blue)
        // a: [1, 5, 8, 12,             25, 30] | 6
        // b: [1, 5, 8,     20, 21, 22]         | 6
        let d = DBall {
            rball: [1, 5, 8, 20, 21, 22],
            bball: 6,
            magnification: 1,
        };
        let dd = DBallBit::from_dball(&d);
        let shared_one = aa.shared_ones(&dd);
        assert_eq!(shared_one, 4);

        let ham = aa.hamming_distance(&dd);
        assert_eq!(ham, 6);
        assert_eq!(aa.euclidean_distance(&dd), (ham as f64).sqrt());
        assert!((aa.cosine_distance(&dd) - (1.0 - shared_one as f64 / 7.0)).abs() < 1e-12);
    }
}
