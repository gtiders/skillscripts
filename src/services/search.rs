use crate::model::Skill;
use fuzzy_matcher::FuzzyMatcher;

pub(crate) fn fuzzy_search<'a>(skills: &'a [Skill], query: &str) -> Vec<(&'a Skill, i64)> {
    let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();

    let mut results: Vec<_> = skills
        .iter()
        .filter_map(|skill| {
            let tags = skill.tags.join(" ");
            let name_score = matcher.fuzzy_match(&skill.name, query).unwrap_or(0);
            let description_score = matcher.fuzzy_match(&skill.description, query).unwrap_or(0) / 2;
            let tags_score = matcher.fuzzy_match(&tags, query).unwrap_or(0) / 2;
            let max_score = [name_score, description_score, tags_score]
                .into_iter()
                .max()
                .unwrap_or(0);

            (max_score > 0).then_some((skill, max_score))
        })
        .collect();

    results.sort_by(|left, right| {
        right
            .1
            .cmp(&left.1)
            .then_with(|| left.0.name.cmp(&right.0.name))
    });
    results
}
