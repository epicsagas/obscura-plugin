use obscura_plugin::install;

// ── transform_skill ────────────────────────────────────────────────────────

const SAMPLE_SKILL: &str = r#"---
name: test-skill
description: A test skill
---

Skill content here.
"#;

#[test]
fn skill_no_transform_for_opencode() {
    assert_eq!(
        install::transform_skill(SAMPLE_SKILL, "opencode"),
        SAMPLE_SKILL
    );
}

#[test]
fn skill_no_transform_for_cline() {
    assert_eq!(
        install::transform_skill(SAMPLE_SKILL, "cline"),
        SAMPLE_SKILL
    );
}

#[test]
fn skill_cursor_adds_frontmatter() {
    let result = install::transform_skill(SAMPLE_SKILL, "cursor");
    assert!(result.contains("globs:"));
    assert!(result.contains("alwaysApply: true"));
    assert!(result.contains("Skill content here."));
}

// ── ALL_TOOLS registry ────────────────────────────────────────────────────

#[test]
fn all_tools_has_expected_count() {
    assert_eq!(install::ALL_TOOLS.len(), 3);
}

#[test]
fn all_tools_contains_known_ids() {
    let ids: Vec<&str> = install::ALL_TOOLS.iter().map(|(id, _)| *id).collect();
    assert!(ids.contains(&"cursor"));
    assert!(ids.contains(&"opencode"));
    assert!(ids.contains(&"cline"));
}

#[test]
fn all_tools_have_names() {
    for (_, name) in install::ALL_TOOLS {
        assert!(!name.is_empty(), "tool name should not be empty");
    }
}
