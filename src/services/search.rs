use crate::model::Skill;
use fuzzy_matcher::FuzzyMatcher;

pub(crate) fn fuzzy_search<'a>(skills: &'a [Skill], query: &str) -> Vec<(&'a Skill, i64)> {
    let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();

    let mut results: Vec<_> = skills
        .iter()
        .filter_map(|skill| {
            let tags = skill.tags.join(" ");
            let name_score = matcher.fuzzy_match(&skill.name, query).unwrap_or(0);
            let tags_score = matcher.fuzzy_match(&tags, query).unwrap_or(0);
            let description_score = matcher.fuzzy_match(&skill.description, query).unwrap_or(0);
            let best_score = name_score.max(tags_score).max(description_score);
            let priority = match (name_score > 0, tags_score > 0, description_score > 0) {
                (true, _, _) => 3,
                (false, true, _) => 2,
                (false, false, true) => 1,
                (false, false, false) => 0,
            };

            (best_score > 0).then_some((skill, priority, name_score, tags_score, description_score))
        })
        .collect();

    results.sort_by(|left, right| {
        right
            .1
            .cmp(&left.1)
            .then_with(|| right.2.cmp(&left.2))
            .then_with(|| right.3.cmp(&left.3))
            .then_with(|| right.4.cmp(&left.4))
            .then_with(|| left.0.name.cmp(&right.0.name))
    });
    results
        .into_iter()
        .map(
            |(skill, _priority, name_score, tags_score, description_score)| {
                (skill, name_score.max(tags_score).max(description_score))
            },
        )
        .collect()
}
