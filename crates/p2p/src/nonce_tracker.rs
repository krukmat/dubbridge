use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NonceCollision;

#[derive(Debug, Default)]
pub(crate) struct NonceTracker {
    seen: HashSet<[u8; 12]>,
}

impl NonceTracker {
    pub(crate) fn register(&mut self, nonce: [u8; 12]) -> Result<(), NonceCollision> {
        if self.seen.insert(nonce) {
            Ok(())
        } else {
            Err(NonceCollision)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hp_t2c_r2_distinct_nonces_register() {
        let mut tracker = NonceTracker::default();

        assert_eq!(tracker.register([1; 12]), Ok(()));
        assert_eq!(tracker.register([2; 12]), Ok(()));
    }

    #[test]
    fn ec_t2c_r2_duplicate_nonce_is_rejected() {
        let mut tracker = NonceTracker::default();
        let nonce = [7; 12];

        assert_eq!(tracker.register(nonce), Ok(()));
        assert_eq!(tracker.register(nonce), Err(NonceCollision));
    }
}
