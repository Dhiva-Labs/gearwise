//! Coalesces rapid updates (slider drags, painting) so the device sees the
//! final value once instead of every intermediate one.

use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};

pub struct Debouncer<K, V> {
    delay: Duration,
    items: HashMap<K, (V, Instant)>,
}

impl<K: Eq + Hash + Clone, V> Debouncer<K, V> {
    pub fn new(delay: Duration) -> Self {
        Self { delay, items: HashMap::new() }
    }

    /// Queue `value` under `key`, restarting its timer. An already queued
    /// value is folded into the new one with `merge(old, new)`.
    pub fn push(&mut self, key: K, value: V, now: Instant, merge: impl FnOnce(V, V) -> V) {
        let value = match self.items.remove(&key) {
            Some((old, _)) => merge(old, value),
            None => value,
        };
        self.items.insert(key, (value, now));
    }

    /// Remove and return whatever is queued under `key`.
    pub fn take(&mut self, key: &K) -> Option<V> {
        self.items.remove(key).map(|(v, _)| v)
    }

    /// Remove and return every entry whose timer has run out.
    pub fn take_due(&mut self, now: Instant) -> Vec<(K, V)> {
        let due: Vec<K> = self
            .items
            .iter()
            .filter(|(_, (_, at))| now.saturating_duration_since(*at) >= self.delay)
            .map(|(k, _)| k.clone())
            .collect();
        due.into_iter().filter_map(|k| self.items.remove(&k).map(|(v, _)| (k, v))).collect()
    }

    /// Time until the earliest entry is due, if anything is queued.
    pub fn next_due(&self, now: Instant) -> Option<Duration> {
        self.items
            .values()
            .map(|(_, at)| self.delay.saturating_sub(now.saturating_duration_since(*at)))
            .min()
    }

    /// Number of queued entries whose key satisfies `pred`.
    pub fn count_where(&self, pred: impl Fn(&K) -> bool) -> usize {
        self.items.keys().filter(|k| pred(k)).count()
    }

    /// Drop queued entries whose key fails `keep`.
    pub fn retain(&mut self, keep: impl Fn(&K) -> bool) {
        self.items.retain(|k, _| keep(k));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MS: fn(u64) -> Duration = Duration::from_millis;

    #[test]
    fn value_is_held_until_quiet_for_the_delay() {
        let t0 = Instant::now();
        let mut d = Debouncer::new(MS(150));
        d.push("dpi", 1, t0, |_, n| n);
        assert!(d.take_due(t0 + MS(100)).is_empty());
        d.push("dpi", 2, t0 + MS(100), |_, n| n);
        // Timer restarted by the second push.
        assert!(d.take_due(t0 + MS(200)).is_empty());
        assert_eq!(d.take_due(t0 + MS(250)), vec![("dpi", 2)]);
        assert_eq!(d.next_due(t0 + MS(250)), None);
    }

    #[test]
    fn next_due_reports_the_remaining_time() {
        let t0 = Instant::now();
        let mut d: Debouncer<&str, i32> = Debouncer::new(MS(150));
        assert_eq!(d.next_due(t0), None);
        d.push("a", 1, t0, |_, n| n);
        assert_eq!(d.next_due(t0 + MS(50)), Some(MS(100)));
        assert_eq!(d.next_due(t0 + MS(400)), Some(MS(0)));
    }

    #[test]
    fn merge_combines_queued_values() {
        let t0 = Instant::now();
        let mut d = Debouncer::new(MS(10));
        d.push("keys", vec![1], t0, |mut a, b| {
            a.extend(b);
            a
        });
        d.push("keys", vec![2], t0, |mut a, b| {
            a.extend(b);
            a
        });
        assert_eq!(d.take(&"keys"), Some(vec![1, 2]));
        assert_eq!(d.take(&"keys"), None);
    }

    #[test]
    fn keys_are_independent() {
        let t0 = Instant::now();
        let mut d = Debouncer::new(MS(10));
        d.push(1, 'a', t0, |_, n| n);
        d.push(2, 'b', t0 + MS(5), |_, n| n);
        assert_eq!(d.count_where(|k| *k == 1), 1);
        assert_eq!(d.take_due(t0 + MS(10)), vec![(1, 'a')]);
        assert_eq!(d.take_due(t0 + MS(15)), vec![(2, 'b')]);
    }
}
