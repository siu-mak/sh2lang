/// Dependency-free suggestion utility for "did you mean …?" diagnostics.
/// Uses Levenshtein edit distance with deterministic ordering.

/// Compute Levenshtein edit distance between two strings.
fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let n = a.len();
    let m = b.len();

    let mut prev = (0..=m).collect::<Vec<_>>();
    let mut curr = vec![0; m + 1];

    for i in 1..=n {
        curr[0] = i;
        for j in 1..=m {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = std::cmp::min(
                std::cmp::min(prev[j] + 1, curr[j - 1] + 1),
                prev[j - 1] + cost,
            );
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[m]
}

/// Return the single best suggestion from `candidates` for `input`,
/// or `None` if nothing is close enough.
///
/// Threshold: `max(1, min(2, input.len() / 2))`.
/// Candidates are sorted before scoring for deterministic results.
pub fn suggest(input: &str, candidates: &[&str]) -> Option<String> {
    if candidates.is_empty() {
        return None;
    }

    let threshold = std::cmp::max(1, std::cmp::min(2, input.len() / 2));

    let mut sorted: Vec<&str> = candidates.to_vec();
    sorted.sort();

    let mut best: Option<(usize, &str)> = None;
    for &c in &sorted {
        let d = levenshtein(input, c);
        if d <= threshold {
            match best {
                None => best = Some((d, c)),
                Some((bd, _)) if d < bd => best = Some((d, c)),
                _ => {} // equal distance: keep first (lexicographic, since sorted)
            }
        }
    }

    best.map(|(_, s)| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match_not_suggested() {
        // Exact match has distance 0, but threshold max(1,..) = 1, so distance 0 passes.
        // However, in practice we never call suggest() with the correct name.
        assert_eq!(suggest("fs", &["fs", "foo"]), Some("fs".to_string()));
    }

    #[test]
    fn test_close_match() {
        assert_eq!(suggest("fx", &["fs", "bar", "baz"]), Some("fs".to_string()));
    }

    #[test]
    fn test_no_match_too_far() {
        assert_eq!(suggest("does_not_exist", &["greet", "sum", "helper"]), None);
    }

    #[test]
    fn test_empty_candidates() {
        assert_eq!(suggest("fx", &[]), None);
    }

    #[test]
    fn test_single_char_input() {
        // "f" with threshold max(1, min(2, 0)) = 1
        assert_eq!(suggest("f", &["fs", "bar"]), Some("fs".to_string()));
    }

    #[test]
    fn test_deterministic_order() {
        // Both "ab" and "ac" are distance 1 from "aa"; "ab" wins lexicographically
        assert_eq!(suggest("aa", &["ac", "ab"]), Some("ab".to_string()));
    }

    #[test]
    fn test_near_miss_function() {
        assert_eq!(suggest("gret", &["greet", "sum", "helper"]), Some("greet".to_string()));
    }
}
