//! Script templates for mofa-cast
//!
//! Pre-formatted podcast script templates for quick start.

use serde::{Deserialize, Serialize};

/// Script template type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TemplateType {
    TwoPersonInterview,
    ThreePersonDiscussion,
    Narrative,
}

impl TemplateType {
    /// Get display name
    pub fn display_name(&self) -> &str {
        match self {
            TemplateType::TwoPersonInterview => "2-Person Interview",
            TemplateType::ThreePersonDiscussion => "3-Person Discussion",
            TemplateType::Narrative => "Narrative / Storytelling",
        }
    }

    /// Get description
    pub fn description(&self) -> &str {
        match self {
            TemplateType::TwoPersonInterview => "Host and guest discussion format",
            TemplateType::ThreePersonDiscussion => "Host with two guests panel format",
            TemplateType::Narrative => "Single narrator storytelling format",
        }
    }
}

/// Script template with content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptTemplate {
    /// Template type
    pub template_type: TemplateType,
    /// Template content
    pub content: String,
}

impl ScriptTemplate {
    /// Create a new template
    pub fn new(template_type: TemplateType) -> Self {
        let content = match template_type {
            TemplateType::TwoPersonInterview => Self::two_person_template(),
            TemplateType::ThreePersonDiscussion => Self::three_person_template(),
            TemplateType::Narrative => Self::narrative_template(),
        };

        Self { template_type, content }
    }

    /// Two-person interview template
    fn two_person_template() -> String {
        r#"host: Welcome to today's episode! Today we're discussing [TOPIC].

guest1: Thanks for having me! I'm excited to be here.

host: Let's start with the basics. Can you tell us a bit about [TOPIC]?

guest1: Absolutely. [TOPIC] is really about [EXPLANATION].

host: That's fascinating. What are some of the key challenges?

guest1: One of the biggest challenges is [CHALLENGE].

host: How do you see this evolving in the future?

guest1: I think we'll see [PREDICTION].

host: That's really interesting. Before we wrap up, any final thoughts?

guest1: I'd encourage your listeners to [CALL_TO_ACTION].

host: Great! Thanks for joining us today. Until next time!"#.to_string()
    }

    /// Three-person discussion template
    fn three_person_template() -> String {
        r#"host: Welcome back! Today we have a special panel discussion about [TOPIC]. I'm joined by guest1 and guest2.

guest1: Great to be here!

guest2: Excited to discuss this important topic.

host: Let's start with you, guest1. What's your perspective on [TOPIC]?

guest1: I think [TOPIC] is crucial because [REASON_1].

guest2: I agree, and I'd add that [REASON_2] is also important.

host: Interesting points. What about the challenges?

guest1: The main challenge I see is [CHALLENGE_1].

guest2: Yes, and we also need to consider [CHALLENGE_2].

host: How can we address these challenges?

guest1: One approach is [SOLUTION_1].

guest2: I'd also suggest [SOLUTION_2].

host: What does the future look like for [TOPIC]?

guest1: I'm optimistic about [FUTURE_1].

guest2: And I think we'll see [FUTURE_2].

host: Any final thoughts for our listeners?

guest1: I'd say [ADVICE_1].

guest2: And I'd add [ADVICE_2].

host: Thank you both! This has been a great discussion."#.to_string()
    }

    /// Narrative template
    fn narrative_template() -> String {
        r#"host: Today, I want to tell you a story about [TOPIC].

host: It all began when [BEGINNING].

host: What happened next was unexpected. [MIDDLE_1].

host: But then, things got even more interesting. [MIDDLE_2].

host: The turning point came when [CLIMAX].

host: After that, everything changed. [RESOLUTION].

host: Looking back, the lessons learned were [LESSON_1] and [LESSON_2].

host: This story teaches us that [THEME].

host: Thanks for listening. Join me next time for another story!"#.to_string()
    }

    /// Get all available templates
    pub fn all_templates() -> Vec<Self> {
        vec![
            Self::new(TemplateType::TwoPersonInterview),
            Self::new(TemplateType::ThreePersonDiscussion),
            Self::new(TemplateType::Narrative),
        ]
    }

    /// Get template by type
    pub fn get_template(template_type: TemplateType) -> Self {
        Self::new(template_type)
    }
}
