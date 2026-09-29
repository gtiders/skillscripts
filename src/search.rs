use crate::fuzzy;
use crate::registry::{Skill, display_path};

const PHRASE_BONUS: i64 = 30_000;
const REQUESTED_TAG_BONUS: i64 = 12_000;
const COVERAGE_BONUS: i64 = 10_000;

#[derive(Debug)]
pub(crate) struct SearchMatch<'a> {
    pub(crate) skill: &'a Skill,
    pub(crate) score: i64,
}

pub(crate) fn script_name(skill: &Skill) -> String {
    skill
        .path
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| skill.path.to_string_lossy().into_owned())
}

pub(crate) fn script_search_text(skill: &Skill) -> String {
    format!(
        "{} {} {} {} {}",
        skill.name,
        display_path(&skill.path),
        skill.command,
        skill.comment.as_deref().unwrap_or_default(),
        skill.tags.join(" ")
    )
}

pub(crate) fn search_skills<'a>(
    skills: &'a [Skill],
    query: Option<&str>,
    tags: &[String],
    limit: usize,
) -> Vec<SearchMatch<'a>> {
    let query = query.map(str::trim).filter(|value| !value.is_empty());
    let terms = query.map(query_terms).unwrap_or_default();
    let mut matches = skills
        .iter()
        .filter_map(|skill| {
            let (query_score, query_matched) = score_query(skill, query, &terms);
            let (tag_score, tag_matched) = score_requested_tags(skill, tags);
            (query_matched || tag_matched).then_some(SearchMatch {
                skill,
                score: query_score + tag_score,
            })
        })
        .collect::<Vec<_>>();

    matches.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.skill.name.cmp(&right.skill.name))
    });
    matches.truncate(limit);
    matches
}

fn score_query(skill: &Skill, query: Option<&str>, terms: &[String]) -> (i64, bool) {
    let Some(query) = query else {
        return (0, false);
    };
    let query_lower = query.to_lowercase();
    let name = script_name(skill);
    let path = display_path(&skill.path);
    let comment = skill.comment.as_deref().unwrap_or_default();
    let tag_text = skill.tags.join(" ");
    let fields = [
        (name.as_str(), 6_000),
        (comment, 4_000),
        (tag_text.as_str(), 8_000),
        (path.as_str(), 2_000),
        (skill.command.as_str(), 1_000),
    ];

    let mut score = fields
        .iter()
        .filter(|(field, _)| field.to_lowercase().contains(&query_lower))
        .map(|(_, weight)| PHRASE_BONUS + weight)
        .max()
        .unwrap_or_default();
    let mut matched_terms = 0usize;
    for term in terms {
        let term_lower = term.to_lowercase();
        let exact = fields
            .iter()
            .filter(|(field, _)| field.to_lowercase().contains(&term_lower))
            .map(|(_, weight)| *weight)
            .max();
        if let Some(weight) = exact {
            matched_terms += 1;
            score += weight;
            continue;
        }

        let fuzzy = fields
            .iter()
            .filter_map(|(field, weight)| {
                fuzzy::score(field, term)
                    .filter(|value| *value >= term.chars().count() as i64 * 8)
                    .map(|value| value + weight / 4)
            })
            .max();
        if let Some(value) = fuzzy {
            matched_terms += 1;
            score += value;
        }
    }

    if !terms.is_empty() && matched_terms > 0 {
        score += COVERAGE_BONUS * matched_terms as i64 / terms.len() as i64;
    }
    let phrase_matched = score >= PHRASE_BONUS;
    if !phrase_matched && matched_terms == 0 {
        let fuzzy = fuzzy::score(&script_search_text(skill), query);
        if let Some(value) = fuzzy.filter(|value| *value >= query.chars().count() as i64 * 8) {
            return (value, true);
        }
    }
    (score, phrase_matched || matched_terms > 0)
}

fn score_requested_tags(skill: &Skill, requested: &[String]) -> (i64, bool) {
    let mut matched = 0usize;
    let mut score = 0;
    for requested in requested {
        let requested = requested.trim();
        if skill
            .tags
            .iter()
            .any(|tag| tag.eq_ignore_ascii_case(requested))
        {
            matched += 1;
            score += REQUESTED_TAG_BONUS;
        } else if skill.tags.iter().any(|tag| {
            tag.to_lowercase().contains(&requested.to_lowercase())
                || requested.to_lowercase().contains(&tag.to_lowercase())
        }) {
            matched += 1;
            score += REQUESTED_TAG_BONUS / 2;
        }
    }
    (score, matched > 0)
}

fn query_terms(query: &str) -> Vec<String> {
    query
        .split(|character: char| {
            !character.is_alphanumeric() && character != '_' && character != '-'
        })
        .map(str::trim)
        .filter(|term| term.chars().count() >= 2)
        .filter(|term| !is_stop_word(term))
        .map(ToOwned::to_owned)
        .collect()
}

fn is_stop_word(term: &str) -> bool {
    matches!(
        term.to_ascii_lowercase().as_str(),
        "a" | "an"
            | "the"
            | "to"
            | "for"
            | "with"
            | "and"
            | "or"
            | "of"
            | "in"
            | "on"
            | "tool"
            | "script"
            | "need"
            | "want"
            | "find"
            | "use"
            | "please"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::ScriptName;
    use std::path::PathBuf;
    use std::str::FromStr;

    fn skill(id: &str, name: &str, comment: &str, tags: &[&str]) -> Skill {
        Skill {
            name: ScriptName::from_str(id).unwrap(),
            path: PathBuf::from(name),
            command: "python {{path}}".to_string(),
            comment: Some(comment.to_string()),
            tags: tags.iter().map(|tag| tag.to_string()).collect(),
        }
    }

    #[test]
    fn fuzzy_search_orders_matches() {
        let skills = vec![
            skill("notes", "notes.py", "Manage notes", &["text"]),
            skill(
                "markdown",
                "markdown-pdf.py",
                "Convert Markdown to PDF",
                &["pdf"],
            ),
        ];
        let matches = search_skills(&skills, Some("markdown pdf"), &[], 5);
        assert_eq!(
            matches[0].skill.name,
            ScriptName::from_str("markdown").unwrap()
        );
    }

    #[test]
    fn tags_are_optional_soft_signals_instead_of_hard_filters() {
        let skills = vec![
            skill("pdf", "pdf.py", "Convert Markdown to PDF", &["PDF"]),
            skill("image", "image.py", "Convert PNG images", &["image"]),
        ];
        assert_eq!(
            search_skills(&skills, Some("markdown pdf"), &[], 5)[0]
                .skill
                .name,
            ScriptName::from_str("pdf").unwrap()
        );
        assert_eq!(
            search_skills(&skills, Some("markdown pdf"), &["image".to_string()], 5)[0]
                .skill
                .name,
            ScriptName::from_str("pdf").unwrap()
        );
    }

    #[test]
    fn natural_language_query_recalls_scripts_without_tags() {
        let skills = vec![skill(
            "render",
            "render.py",
            "Convert Markdown documents into PDF files",
            &[],
        )];
        assert_eq!(
            search_skills(
                &skills,
                Some("please find a tool to convert markdown to pdf"),
                &[],
                5
            )
            .len(),
            1
        );
    }

    #[test]
    fn limit_returns_only_the_five_highest_value_matches() {
        let skills = (1..=8)
            .map(|id| {
                skill(
                    &format!("pdf_{id}"),
                    &format!("pdf-{id}.py"),
                    "Create PDF documents",
                    &[],
                )
            })
            .collect::<Vec<_>>();
        let matches = search_skills(&skills, Some("pdf"), &[], 5);
        assert_eq!(matches.len(), 5);
        assert_eq!(
            matches[0].skill.name,
            ScriptName::from_str("pdf_1").unwrap()
        );
    }
}
