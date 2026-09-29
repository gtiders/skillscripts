/// Match every query character in order and return a score and character positions.
/// Adjacent characters and word boundaries rank above dispersed matches.
pub(crate) fn match_indices(haystack: &str, query: &str) -> Option<(i64, Vec<usize>)> {
    let text: Vec<char> = haystack.chars().collect();
    let needle: Vec<char> = query.chars().collect();
    if needle.is_empty() {
        return Some((0, Vec::new()));
    }
    if text.is_empty() {
        return None;
    }

    let mut scores = vec![vec![i64::MIN; text.len()]; needle.len()];
    let mut previous = vec![vec![None; text.len()]; needle.len()];
    for (row, wanted) in needle.iter().enumerate() {
        for (column, found) in text.iter().enumerate() {
            if !found.eq_ignore_ascii_case(wanted)
                && found.to_lowercase().to_string() != wanted.to_lowercase().to_string()
            {
                continue;
            }
            let boundary = column == 0
                || !text[column - 1].is_alphanumeric()
                || (found.is_uppercase() && text[column - 1].is_lowercase());
            let bonus = 10 + if boundary { 8 } else { 0 };
            if row == 0 {
                scores[row][column] = bonus + (10 - column as i64 / 4).max(0);
                continue;
            }
            for prior in 0..column {
                let old = scores[row - 1][prior];
                if old == i64::MIN {
                    continue;
                }
                let gap = column - prior - 1;
                let score = old + bonus + if gap == 0 { 12 } else { -(gap.min(20) as i64) };
                if score > scores[row][column] {
                    scores[row][column] = score;
                    previous[row][column] = Some(prior);
                }
            }
        }
    }

    let (mut column, &score) = scores
        .last()?
        .iter()
        .enumerate()
        .max_by_key(|(_, value)| *value)?;
    if score == i64::MIN {
        return None;
    }
    let mut indices = vec![0; needle.len()];
    for row in (0..needle.len()).rev() {
        indices[row] = column;
        if row > 0 {
            column = previous[row][column]?;
        }
    }
    Some((score, indices))
}

pub(crate) fn score(haystack: &str, query: &str) -> Option<i64> {
    match_indices(haystack, query).map(|(score, _)| score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_in_order_and_prefers_adjacent_boundaries() {
        assert_eq!(
            match_indices("convert_ase", "ASE").unwrap().1,
            vec![8, 9, 10]
        );
        assert!(score("ase", "ase") > score("a_s_e", "ase"));
        assert!(score("abc", "acb").is_none());
    }
}
