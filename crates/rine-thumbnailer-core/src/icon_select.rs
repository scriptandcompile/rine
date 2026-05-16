//! Icon selection logic: pick the best icon variant for the requested size.
//!
//! Selection strategy:
//! 1. Prefer icons with width >= requested size (avoid upscaling).
//! 2. Among those, choose the smallest (minimise unnecessary downscaling).
//! 3. Tiebreak by bit depth (higher is better quality).
//! 4. If no icon is large enough, fall back to the largest available,
//!    tiebroken by bit depth.

use crate::pe_resources::GroupIconEntry;

pub fn select_best(entries: &[GroupIconEntry], size_px: u32) -> Option<&GroupIconEntry> {
    if entries.is_empty() {
        return None;
    }

    // Candidates that are at least as large as the target.
    let mut candidates: Vec<&GroupIconEntry> =
        entries.iter().filter(|e| e.width >= size_px).collect();

    if candidates.is_empty() {
        // Fall back: all icons are smaller than requested – take the largest.
        candidates = entries.iter().collect();
        return candidates
            .into_iter()
            .max_by_key(|e| (e.width, e.bit_count));
    }

    // Among candidates ≥ requested size, prefer smallest (then highest bit depth).
    candidates
        .into_iter()
        .min_by_key(|e| (e.width, std::cmp::Reverse(e.bit_count)))
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(width: u32, bit_count: u16, id: u16) -> GroupIconEntry {
        GroupIconEntry { width, height: width, bit_count, id }
    }

    #[test]
    fn selects_exact_match() {
        let entries = vec![entry(16, 32, 1), entry(32, 32, 2), entry(256, 32, 3)];
        let best = select_best(&entries, 32).unwrap();
        assert_eq!(best.id, 2);
    }

    #[test]
    fn selects_next_larger() {
        let entries = vec![entry(16, 32, 1), entry(64, 32, 2), entry(256, 32, 3)];
        let best = select_best(&entries, 32).unwrap();
        assert_eq!(best.id, 2);
    }

    #[test]
    fn falls_back_to_largest_when_all_smaller() {
        let entries = vec![entry(16, 32, 1), entry(32, 32, 2)];
        let best = select_best(&entries, 48).unwrap();
        assert_eq!(best.id, 2);
    }

    #[test]
    fn tiebreaks_by_bit_depth() {
        let entries = vec![entry(32, 4, 1), entry(32, 32, 2), entry(32, 8, 3)];
        let best = select_best(&entries, 32).unwrap();
        assert_eq!(best.id, 2); // highest bit depth wins
    }

    #[test]
    fn returns_none_for_empty() {
        assert!(select_best(&[], 32).is_none());
    }
}
