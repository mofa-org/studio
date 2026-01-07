// Policy module for conference controller
// This module defines traits and implementations for customizable control policies

pub mod unified_ratio;

pub use unified_ratio::{PatternParser, PolicyPattern, UnifiedRatioPolicy, Weight};

/// Core trait that defines the interface for all control policies
///
/// Implementations of this trait determine which participant should speak next
/// based on configurable logic (round-robin, sequential, ratio-based, priority, etc.)
pub trait Policy: Send + Sync {
    /// Update the word count for a speaker
    ///
    /// This is called when a participant has spoken words, allowing the policy
    /// to track speaking time and make informed decisions
    fn update_word_count(&mut self, speaker: &str, word_count: usize);

    /// Determine the next speaker based on the policy's logic
    ///
    /// Returns the name of the participant who should speak next,
    /// or None if no participant is available
    fn determine_next_speaker(&mut self) -> Option<String>;

    /// Check if all participants have completed in the current round
    ///
    /// Returns true if all participants have spoken in the current cycle
    fn all_participants_completed(&self) -> bool;

    /// Reset round completion tracking
    ///
    /// Called when starting a new conversation round
    fn reset_round_tracking(&mut self);

    /// Increment cycle counter
    ///
    /// Called when all participants complete a round
    fn increment_cycle(&mut self);

    /// Get current cycle number
    ///
    /// Returns the current cycle based on policy mode
    fn get_current_cycle(&self) -> usize;
}

#[cfg(test)]
mod tests {
    mod unified_ratio_test;
}
