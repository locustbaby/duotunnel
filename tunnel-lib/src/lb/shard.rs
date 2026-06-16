use std::hash::{Hash, Hasher};

use crate::lb::inflight::pick_p2c_inflight;

pub const DEFAULT_P2C_THRESHOLD: usize = 32;
pub const DEFAULT_P2C_MAX_RETRIES: usize = 3;

pub fn stable_shard_index<T: Hash + ?Sized>(value: &T, shard_count: usize) -> usize {
    let shard_count = shard_count.max(1);
    if shard_count == 1 {
        return 0;
    }

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    (hasher.finish() as usize) % shard_count
}

pub fn pick_p2c_inflight_owned<T, H, I>(items: &[T], is_healthy: H, inflight: I) -> Option<T>
where
    T: Clone,
    H: Fn(&T) -> bool,
    I: Fn(&T) -> usize,
{
    pick_p2c_inflight(
        items,
        DEFAULT_P2C_THRESHOLD,
        DEFAULT_P2C_MAX_RETRIES,
        is_healthy,
        inflight,
    )
    .cloned()
}

pub fn pick_from_preferred_shards<S, T, F>(
    shards: &[S],
    preferred_shard: usize,
    pick_from_shard: F,
) -> Option<T>
where
    F: Fn(&S) -> Option<T>,
{
    if shards.is_empty() {
        return None;
    }

    let preferred_shard = preferred_shard % shards.len();
    for offset in 0..shards.len() {
        if let Some(item) = pick_from_shard(&shards[(preferred_shard + offset) % shards.len()]) {
            return Some(item);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{pick_from_preferred_shards, pick_p2c_inflight_owned, stable_shard_index};

    #[derive(Clone)]
    struct Candidate {
        id: usize,
        healthy: bool,
        load: usize,
    }

    #[test]
    fn stable_shard_index_handles_small_shard_counts() {
        assert_eq!(stable_shard_index(&"group-a", 0), 0);
        assert_eq!(stable_shard_index(&"group-a", 1), 0);
    }

    #[test]
    fn preferred_shard_is_tried_before_fallback() {
        let shards = vec![
            vec![Candidate {
                id: 1,
                healthy: true,
                load: 10,
            }],
            vec![Candidate {
                id: 2,
                healthy: true,
                load: 1,
            }],
        ];

        let chosen = pick_from_preferred_shards(&shards, 1, |shard| {
            pick_p2c_inflight_owned(shard, |item| item.healthy, |item| item.load)
        })
        .expect("candidate from preferred shard");

        assert_eq!(chosen.id, 2);
    }

    #[test]
    fn falls_back_to_later_shards_when_preferred_is_unhealthy() {
        let shards = vec![
            vec![Candidate {
                id: 1,
                healthy: false,
                load: 0,
            }],
            vec![Candidate {
                id: 2,
                healthy: true,
                load: 5,
            }],
        ];

        let chosen = pick_from_preferred_shards(&shards, 0, |shard| {
            pick_p2c_inflight_owned(shard, |item| item.healthy, |item| item.load)
        })
        .expect("candidate from fallback shard");

        assert_eq!(chosen.id, 2);
    }
}
