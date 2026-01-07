use crate::policies::{PatternParser, PolicyPattern, UnifiedRatioPolicy, Weight, Policy};

#[test]
fn test_parse_sequential_pattern() {
    let pattern_str = "[judge → defense → prosecution]";
    let result = PatternParser::parse(pattern_str).unwrap();

    match result {
        PolicyPattern::Sequential {
            participants,
            loop_forever,
        } => {
            assert_eq!(participants, vec!["judge", "defense", "prosecution"]);
            assert!(loop_forever);
        }
        _ => panic!("Expected sequential pattern"),
    }
}

#[test]
fn test_parse_sequential_without_brackets() {
    let pattern_str = "judge → defense → prosecution";
    let result = PatternParser::parse(pattern_str).unwrap();

    match result {
        PolicyPattern::Sequential { participants, .. } => {
            assert_eq!(participants, vec!["judge", "defense", "prosecution"]);
        }
        _ => panic!("Expected sequential pattern"),
    }
}

#[test]
fn test_parse_ratio_priority_pattern_with_ratios() {
    let pattern_str = "[(judge, 2), (defense, 1), (prosecution, 1)]";
    let result = PatternParser::parse(pattern_str).unwrap();

    match result {
        PolicyPattern::RatioPriority {
            participants,
            weights,
        } => {
            assert_eq!(participants, vec!["judge", "defense", "prosecution"]);
            assert_eq!(weights.len(), 3);

            if let Weight::Ratio(r) = &weights[0] {
                assert_eq!(*r, 2.0);
            } else {
                panic!("Expected ratio weight");
            }

            if let Weight::Ratio(r) = &weights[1] {
                assert_eq!(*r, 1.0);
            } else {
                panic!("Expected ratio weight");
            }
        }
        _ => panic!("Expected ratio priority pattern"),
    }
}

#[test]
fn test_parse_ratio_priority_pattern_with_priority() {
    let pattern_str = "[(judge, *), (defense, 1), (prosecution, 1)]";
    let result = PatternParser::parse(pattern_str).unwrap();

    match result {
        PolicyPattern::RatioPriority { weights, .. } => {
            assert_eq!(weights.len(), 3);

            if let Weight::Priority = &weights[0] {
                // Good
            } else {
                panic!("Expected priority weight at position 0");
            }

            if let Weight::Ratio(r) = &weights[1] {
                assert_eq!(*r, 1.0);
            } else {
                panic!("Expected ratio weight at position 1");
            }
        }
        _ => panic!("Expected ratio priority pattern"),
    }
}

#[test]
fn test_parse_simple_ratio_pattern_no_parenthesis() {
    let pattern_str = "[judge, defense, prosecution]";
    let result = PatternParser::parse(pattern_str).unwrap();

    match result {
        PolicyPattern::RatioPriority {
            participants,
            weights,
        } => {
            assert_eq!(participants, vec!["judge", "defense", "prosecution"]);
            assert_eq!(weights.len(), 3);

            // All should have default ratio of 1.0
            for weight in weights {
                if let Weight::Ratio(r) = weight {
                    assert_eq!(r, 1.0);
                } else {
                    panic!("Expected ratio weight");
                }
            }
        }
        _ => panic!("Expected ratio priority pattern"),
    }
}

#[test]
fn test_parse_invalid_pattern_empty() {
    let result = PatternParser::parse("[]");
    assert!(result.is_err());
}

#[test]
fn test_parse_invalid_pattern_invalid_ratio() {
    let result = PatternParser::parse("[(judge, 0)]");
    assert!(result.is_err());
}

#[test]
fn test_policy_sequential() {
    let mut policy = UnifiedRatioPolicy::new();
    policy.configure("[A → B → C]").unwrap();

    // Should cycle through A, B, C, A, B, C, ...
    assert_eq!(policy.determine_next_speaker(), Some("A".to_string()));
    assert_eq!(policy.determine_next_speaker(), Some("B".to_string()));
    assert_eq!(policy.determine_next_speaker(), Some("C".to_string()));
    assert_eq!(policy.determine_next_speaker(), Some("A".to_string()));
    assert_eq!(policy.determine_next_speaker(), Some("B".to_string()));
}

#[test]
fn test_policy_ratio_priority_with_priority() {
    let mut policy = UnifiedRatioPolicy::new();
    policy
        .configure("[(judge, *), (defense, 1), (prosecution, 1)]")
        .unwrap();

    // First speaker should be judge (priority)
    let speaker1 = policy.determine_next_speaker();
    assert_eq!(speaker1, Some("judge".to_string()));

    // Update word counts: judge spoke 100 words
    policy.update_word_count("judge", 100);

    // Next speaker should NOT be judge (can't be same as last)
    // Should be based on ratio of defense:prosecution (1:1)
    let speaker2 = policy.determine_next_speaker();
    assert_ne!(speaker2, Some("judge".to_string()));
    assert!(speaker2 == Some("defense".to_string()) || speaker2 == Some("prosecution".to_string()));
}

#[test]
fn test_policy_ratio_priority_ratio_based() {
    let mut policy = UnifiedRatioPolicy::new();
    policy
        .configure("[(judge, 2), (defense, 1), (prosecution, 1)]")
        .unwrap();

    // Should select speaker based on ratios
    // Initially all have 0 words, so ratio difference is based on weight/total_weight
    let speaker = policy.determine_next_speaker().unwrap();
    println!("First speaker: {}", speaker);

    // Simulate 10 turns
    for i in 0..10 {
        let speaker = policy.determine_next_speaker().unwrap();
        policy.update_word_count(&speaker, 100); // Each speaks 100 words
        println!("Turn {}: {:?}", i, speaker);
    }

    // Check stats - judge should have spoken more
    let stats = policy.get_stats();
    println!("Stats: {}", serde_json::to_string_pretty(&stats).unwrap());

    // judge had weight 2, defense and prosecution had weight 1 each
    // So ratio should be approximately: judge:defense:prosecution = 40:30:30
    if let Some(counts) = stats.get("word_counts").and_then(|v| v.as_object()) {
        let judge_count = counts.get("judge").and_then(|v| v.as_u64()).unwrap_or(0);
        let defense_count = counts.get("defense").and_then(|v| v.as_u64()).unwrap_or(0);
        let prosecution_count = counts.get("prosecution").and_then(|v| v.as_u64()).unwrap_or(0);

        assert!(judge_count > defense_count);
        assert!(judge_count > prosecution_count);
    }
}

#[test]
fn test_policy_simple_ratio_equal_weights() {
    let mut policy = UnifiedRatioPolicy::new();
    policy.configure("[judge, defense, prosecution]").unwrap();

    // All have same ratio (1.0 by default)
    // Should distribute evenly
    for _ in 0..9 {
        let speaker = policy.determine_next_speaker().unwrap();
        policy.update_word_count(&speaker, 100);
    }

    let stats = policy.get_stats();
    if let Some(counts) = stats.get("word_counts").and_then(|v| v.as_object()) {
        let judge_count = counts.get("judge").and_then(|v| v.as_u64()).unwrap_or(0);
        let defense_count = counts.get("defense").and_then(|v| v.as_u64()).unwrap_or(0);
        let prosecution_count = counts.get("prosecution").and_then(|v| v.as_u64()).unwrap_or(0);

        // All should have spoken equal amount (within reason)
        assert_eq!(judge_count, 300);
        assert_eq!(defense_count, 300);
        assert_eq!(prosecution_count, 300);
    }
}

#[test]
fn test_policy_change_pattern() {
    let mut policy = UnifiedRatioPolicy::new();

    // Start with sequential
    policy.configure("[A → B → C]").unwrap();
    assert_eq!(policy.determine_next_speaker(), Some("A".to_string()));

    // Change to ratio-based
    policy.configure("[(A, 1), (B, 1), (C, 1)]").unwrap();
    // Should reset state
    assert_eq!(policy.get_stats()["word_counts"]["A"], 0);
}

#[test]
fn test_policy_validation() {
    let mut policy = UnifiedRatioPolicy::new();

    // Valid patterns
    assert!(policy.configure("[A → B → C]").is_ok());
    assert!(policy.configure("[(A, 1), (B, 2)]").is_ok());
    assert!(policy.configure("[(A, *), (B, 1)]").is_ok());
    assert!(policy.configure("[A, B, C]").is_ok());

    // Invalid patterns
    assert!(PatternParser::parse("[]").is_err());
    assert!(PatternParser::validate(&PolicyPattern::RatioPriority {
        participants: vec![],
        weights: vec![],
    })
    .is_err());
}

#[test]
fn test_stats_generation() {
    let mut policy = UnifiedRatioPolicy::new();
    policy
        .configure("[(judge, 2), (defense, 1), (prosecution, 1)]")
        .unwrap();

    // Update some word counts
    policy.update_word_count("judge", 300);
    policy.update_word_count("defense", 200);
    policy.update_word_count("prosecution", 150);

    let stats = policy.get_stats();

    // Check mode
    assert_eq!(stats["mode"], "ratio_priority");

    // Check participants
    let participants = stats["participants"].as_array().unwrap();
    assert_eq!(participants.len(), 3);

    // Check weights
    let weights = stats["weights"].as_array().unwrap();
    assert_eq!(weights.len(), 3);

    // Check word counts
    assert_eq!(stats["word_counts"]["judge"], 300);
    assert_eq!(stats["word_counts"]["defense"], 200);
    assert_eq!(stats["word_counts"]["prosecution"], 150);
}
