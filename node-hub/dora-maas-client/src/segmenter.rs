/// Segmenter for streaming text that intelligently buffers chunks into meaningful segments.
///
/// The segmenter is designed for real-time text-to-speech applications where sending
/// individual characters or words would overwhelm the TTS system. It buffers incoming
/// text chunks and emits complete segments based on punctuation boundaries.
///
/// # Example
/// ```
/// let mut segmenter = StreamSegmenter::new(10);
/// assert_eq!(segmenter.add_chunk("Hello"), None);
/// assert_eq!(segmenter.add_chunk(" world."), Some("Hello world.".to_string()));
/// ```
pub struct StreamSegmenter {
    buffer: String,
    word_count: usize,
    max_words_without_punctuation: usize,
    max_chars_without_punctuation: usize, // For Chinese text
}

impl StreamSegmenter {
    pub fn new(max_words: usize) -> Self {
        Self {
            buffer: String::new(),
            word_count: 0,
            max_words_without_punctuation: max_words,
            max_chars_without_punctuation: 10, // Strict 10 Chinese character limit for TTS
        }
    }

    /// Add a text chunk to the buffer and return a segment if one is ready.
    ///
    /// The method buffers the incoming chunk and checks if a meaningful segment
    /// can be emitted based on punctuation marks or word count limits.
    ///
    /// # Arguments
    /// * `chunk` - A text chunk received from the streaming API
    ///
    /// # Returns
    /// * `Some(String)` - A complete segment ready for TTS processing
    /// * `None` - No segment ready yet, more buffering needed
    pub fn add_chunk(&mut self, chunk: &str) -> Option<String> {
        // Clean markdown from the chunk before adding to buffer
        let cleaned_chunk = self.clean_markdown(chunk);
        self.buffer.push_str(&cleaned_chunk);

        // Count words in the current buffer
        self.word_count = self.buffer.split_whitespace().count();

        // Check if we should emit a segment
        if self.should_emit_segment() {
            self.emit_segment()
        } else {
            None
        }
    }

    /// Check if we should emit a segment
    fn should_emit_segment(&self) -> bool {
        if self.buffer.is_empty() {
            return false;
        }

        // Check for meaningful punctuation (Chinese and English)
        let punctuation_marks = [
            '。', '！', '？', '；', '：', // Chinese
            '.', '!', '?', ';', ':', // English
            '，', ',',  // Comma (both Chinese and English)
            '\n', // Newline
        ];

        // Check if buffer ends with punctuation
        let has_punctuation = self.buffer.chars().any(|c| punctuation_marks.contains(&c));

        // Count Chinese characters (for Chinese text segmentation)
        let chinese_char_count = self
            .buffer
            .chars()
            .filter(|c| self.is_chinese_char(*c))
            .count();

        // Emit if:
        // 1. We have punctuation, OR
        // 2. We have reached max words without punctuation (for English), OR
        // 3. We have reached max Chinese characters without punctuation
        has_punctuation
            || self.word_count >= self.max_words_without_punctuation
            || chinese_char_count >= self.max_chars_without_punctuation
    }

    /// Check if a character is Chinese
    fn is_chinese_char(&self, c: char) -> bool {
        // Unicode ranges for Chinese characters
        match c {
            // CJK Unified Ideographs
            '\u{4e00}'..='\u{9fff}' => true,
            // CJK Unified Ideographs Extension A
            '\u{3400}'..='\u{4dbf}' => true,
            // CJK Unified Ideographs Extension B
            '\u{20000}'..='\u{2a6df}' => true,
            // CJK Unified Ideographs Extension C
            '\u{2a700}'..='\u{2b73f}' => true,
            // CJK Unified Ideographs Extension D
            '\u{2b740}'..='\u{2b81f}' => true,
            _ => false,
        }
    }

    /// Emit the current buffer as a segment
    fn emit_segment(&mut self) -> Option<String> {
        if self.buffer.is_empty() {
            return None;
        }

        // Find the best split point
        let split_point = self.find_split_point();

        if split_point > 0 {
            // Take the segment up to the split point
            let segment = self.buffer[..split_point].to_string();

            // Keep the rest in the buffer
            self.buffer = self.buffer[split_point..].trim_start().to_string();
            self.word_count = self.buffer.split_whitespace().count();

            Some(segment)
        } else if self.word_count >= self.max_words_without_punctuation {
            // No punctuation found but we've reached max words
            // Emit the entire buffer
            let segment = self.buffer.clone();
            self.buffer.clear();
            self.word_count = 0;
            Some(segment)
        } else {
            // Check if we need to emit due to Chinese character count
            let chinese_char_count = self
                .buffer
                .chars()
                .filter(|c| self.is_chinese_char(*c))
                .count();

            if chinese_char_count >= self.max_chars_without_punctuation {
                // Too many Chinese characters without punctuation
                let segment = self.buffer.clone();
                self.buffer.clear();
                self.word_count = 0;
                Some(segment)
            } else {
                None
            }
        }
    }

    /// Find the best point to split the buffer
    fn find_split_point(&self) -> usize {
        // Count Chinese characters to enforce strict limit
        let chinese_char_count = self
            .buffer
            .chars()
            .filter(|c| self.is_chinese_char(*c))
            .count();

        // Priority punctuation for splitting (sentence endings first)
        let sentence_endings = ['。', '！', '？', '.', '!', '?'];
        let clause_endings = ['；', '：', ';', ':'];
        let soft_breaks = ['，', ',', '\n'];

        // For Chinese text, be more aggressive with segmentation
        if chinese_char_count > 0 {
            // If we're approaching the limit, look for ANY punctuation to split at
            if chinese_char_count >= 8 {
                // Try sentence endings first
                if let Some(pos) = self.find_punctuation_before_limit(&sentence_endings, 10) {
                    return pos + self.char_len_at(pos);
                }
                // Then clause endings
                if let Some(pos) = self.find_punctuation_before_limit(&clause_endings, 10) {
                    return pos + self.char_len_at(pos);
                }
                // Then soft breaks (comma)
                if let Some(pos) = self.find_punctuation_before_limit(&soft_breaks, 10) {
                    return pos + self.char_len_at(pos);
                }
            }
        }

        // For English or mixed text, use the original logic
        // Try to find sentence ending first
        if let Some(pos) = self.find_last_punctuation(&sentence_endings) {
            return pos + self.char_len_at(pos);
        }

        // Then try clause endings
        if let Some(pos) = self.find_last_punctuation(&clause_endings) {
            return pos + self.char_len_at(pos);
        }

        // Finally try soft breaks if we have enough words
        if self.word_count >= 5 || chinese_char_count >= 5 {
            if let Some(pos) = self.find_last_punctuation(&soft_breaks) {
                return pos + self.char_len_at(pos);
            }
        }

        0
    }

    /// Find the last occurrence of any punctuation mark
    fn find_last_punctuation(&self, marks: &[char]) -> Option<usize> {
        self.buffer
            .char_indices()
            .rev()
            .find(|(_, c)| marks.contains(c))
            .map(|(i, _)| i)
    }

    /// Find punctuation within the first N Chinese characters
    fn find_punctuation_before_limit(&self, marks: &[char], char_limit: usize) -> Option<usize> {
        let mut chinese_count = 0;
        let mut last_punct_pos = None;

        for (i, c) in self.buffer.char_indices() {
            if self.is_chinese_char(c) {
                chinese_count += 1;
                if chinese_count > char_limit {
                    // We've exceeded the limit, return the last punctuation we found
                    return last_punct_pos;
                }
            }

            if marks.contains(&c) {
                last_punct_pos = Some(i);
            }
        }

        last_punct_pos
    }

    /// Get the byte length of the character at the given position
    fn char_len_at(&self, byte_pos: usize) -> usize {
        self.buffer[byte_pos..]
            .chars()
            .next()
            .map(|c| c.len_utf8())
            .unwrap_or(1)
    }

    /// Clean markdown formatting from text for TTS processing
    ///
    /// Removes common markdown symbols that would sound awkward when spoken:
    /// - Bold: **text** → text
    /// - Italic: *text* → text  
    /// - Headers: ## text → text
    /// - Code: `code` → code
    /// - Links: [text](url) → text
    ///
    /// # Arguments
    /// * `text` - Raw text chunk that may contain markdown
    ///
    /// # Returns
    /// * `String` - Cleaned text suitable for TTS
    fn clean_markdown(&self, text: &str) -> String {
        let mut result = text.to_string();

        // Remove list markers at the beginning of text
        // Remove "- " or "* " or "+ " at the start
        if result.starts_with("- ") || result.starts_with("* ") || result.starts_with("+ ") {
            result = result[2..].to_string();
        }

        // Remove numbered list markers like "1. " or "12. " at the start
        let trimmed = result.trim_start();
        if let Some(dot_pos) = trimmed.find(". ") {
            if dot_pos > 0 && dot_pos <= 3 {
                // Check if it's a reasonable number (1-999)
                let prefix = &trimmed[..dot_pos];
                if prefix.chars().all(|c| c.is_ascii_digit()) {
                    result = trimmed[dot_pos + 2..].to_string();
                }
            }
        }

        // Remove bold formatting: **text** → text
        result = result.replace("**", "");

        // Remove italic formatting: *text* → text (but preserve Chinese punctuation)
        // Only remove * that are clearly markdown (surrounded by alphanumeric)
        let mut chars: Vec<char> = result.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '*' {
                let prev_is_alnum = i > 0 && chars[i - 1].is_alphanumeric();
                let next_is_alnum = i + 1 < chars.len() && chars[i + 1].is_alphanumeric();

                // Remove * if it's likely markdown (between alphanumeric chars)
                if prev_is_alnum || next_is_alnum {
                    chars.remove(i);
                    continue; // Don't increment i since we removed a character
                }
            }
            i += 1;
        }
        result = chars.into_iter().collect();

        // Remove headers: ## text → text, ### text → text, etc.
        // Simple approach: remove lines starting with # followed by space
        let lines: Vec<&str> = result.lines().collect();
        let mut cleaned_lines = Vec::new();
        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with('#')
                && trimmed.chars().nth(1).map_or(false, |c| c.is_whitespace())
            {
                // Remove the # symbols and leading whitespace
                let cleaned = trimmed.trim_start_matches('#').trim();
                if !cleaned.is_empty() {
                    cleaned_lines.push(cleaned);
                }
            } else {
                cleaned_lines.push(line);
            }
        }
        result = cleaned_lines.join("\n");

        // Remove code blocks: `code` → code
        result = result.replace('`', "");

        // Remove links: [text](url) → text - simple approach
        while let Some(start) = result.find('[') {
            if let Some(middle) = result[start..].find("](") {
                let middle_pos = start + middle;
                if let Some(end) = result[middle_pos..].find(')') {
                    let end_pos = middle_pos + end;
                    let link_text = &result[start + 1..middle_pos];
                    result = result[..start].to_string() + link_text + &result[end_pos + 1..];
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Replace hyphens with spaces in Chinese context (e.g., "2根-土豆" → "2根 土豆")
        // This helps TTS pronounce it more naturally
        result = result.replace('-', " ");

        // Collapse multiple consecutive spaces into single space
        // Preserve leading/trailing spaces since we're processing chunks
        while result.contains("  ") {
            result = result.replace("  ", " ");
        }

        result
    }

    /// Force emit any remaining buffered content.
    ///
    /// This should be called when the stream ends to ensure no text is lost.
    /// After flushing, the buffer is cleared and ready for new content.
    ///
    /// # Returns
    /// * `Some(String)` - The remaining buffered text
    /// * `None` - Buffer was already empty
    pub fn flush(&mut self) -> Option<String> {
        if self.buffer.is_empty() {
            None
        } else {
            let segment = self.buffer.clone();
            self.buffer.clear();
            self.word_count = 0;
            Some(segment)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sentence_segmentation() {
        let mut segmenter = StreamSegmenter::new(10);

        // Add chunks
        assert_eq!(segmenter.add_chunk("Hello"), None);
        assert_eq!(segmenter.add_chunk(" world"), None);
        assert_eq!(segmenter.add_chunk("."), Some("Hello world.".to_string()));

        assert_eq!(segmenter.add_chunk(" How"), None);
        assert_eq!(segmenter.add_chunk(" are"), None);
        assert_eq!(
            segmenter.add_chunk(" you?"),
            Some(" How are you?".to_string())
        );
    }

    #[test]
    fn test_chinese_segmentation() {
        let mut segmenter = StreamSegmenter::new(10);

        assert_eq!(segmenter.add_chunk("你好"), None);
        assert_eq!(segmenter.add_chunk("世界"), None);
        assert_eq!(segmenter.add_chunk("。"), Some("你好世界。".to_string()));

        assert_eq!(segmenter.add_chunk("今天"), None);
        assert_eq!(segmenter.add_chunk("天气"), None);
        assert_eq!(segmenter.add_chunk("很好"), None);
        assert_eq!(
            segmenter.add_chunk("！"),
            Some("今天天气很好！".to_string())
        );
    }

    #[test]
    fn test_max_words() {
        let mut segmenter = StreamSegmenter::new(5);

        // Add exactly 5 words - should trigger emission
        assert_eq!(
            segmenter.add_chunk("one two three four five"),
            Some("one two three four five".to_string())
        );

        // Test with fewer words than limit
        let mut segmenter2 = StreamSegmenter::new(5);
        assert_eq!(segmenter2.add_chunk("one two three"), None);
        assert_eq!(segmenter2.add_chunk(" four"), None);
        // Adding fifth word triggers emission
        assert_eq!(
            segmenter2.add_chunk(" five"),
            Some("one two three four five".to_string())
        );
    }

    #[test]
    fn test_flush() {
        let mut segmenter = StreamSegmenter::new(10);

        assert_eq!(segmenter.add_chunk("Hello world"), None);
        assert_eq!(segmenter.flush(), Some("Hello world".to_string()));
        assert_eq!(segmenter.flush(), None); // Nothing left
    }

    #[test]
    fn test_chinese_character_segmentation() {
        let mut segmenter = StreamSegmenter::new(10); // 10 English words, 20 Chinese chars max

        // Test long Chinese text without punctuation - should segment at 20 characters
        let long_chinese =
            "第三鲜是一道传统的中国菜通常以鸡蛋虾仁和黄瓜为主料因其色香味俱佳而得名这道菜"; // 35 characters

        // Add the text in chunks
        assert_eq!(segmenter.add_chunk("第三鲜是一道传统的中国菜通常以"), None); // 13 chars
        assert_eq!(
            segmenter.add_chunk("鸡蛋虾仁和黄瓜为主料"),
            Some("第三鲜是一道传统的中国菜通常以鸡蛋虾仁和黄瓜为主料".to_string())
        ); // +10 = 23 chars total > 20

        // Test Chinese with punctuation - should segment at punctuation
        let mut segmenter2 = StreamSegmenter::new(10);
        assert_eq!(
            segmenter2.add_chunk("第三鲜是一道传统菜。"),
            Some("第三鲜是一道传统菜。".to_string())
        );
    }

    #[test]
    fn test_markdown_cleaning() {
        let segmenter = StreamSegmenter::new(10);

        // Test list markers
        assert_eq!(
            segmenter.clean_markdown("- 在锅中留少量底油"),
            "在锅中留少量底油"
        );
        assert_eq!(segmenter.clean_markdown("* 放入豆角煸炒"), "放入豆角煸炒");
        assert_eq!(segmenter.clean_markdown("+ 加入调料"), "加入调料");
        assert_eq!(segmenter.clean_markdown("1. 第一步"), "第一步");
        assert_eq!(segmenter.clean_markdown("12. 第十二步"), "第十二步");

        // Test bold markdown
        assert_eq!(segmenter.clean_markdown("**准备材料**"), "准备材料");
        assert_eq!(
            segmenter.clean_markdown("This is **bold** text"),
            "This is bold text"
        );

        // Test italic markdown (be careful with Chinese text)
        assert_eq!(segmenter.clean_markdown("*italic* text"), "italic text");

        // Test headers
        assert_eq!(
            segmenter.clean_markdown("## Section Title"),
            "Section Title"
        );
        assert_eq!(segmenter.clean_markdown("### Subsection"), "Subsection");

        // Test code blocks
        assert_eq!(
            segmenter.clean_markdown("Use `function()` here"),
            "Use function() here"
        );

        // Test links
        assert_eq!(
            segmenter.clean_markdown("Visit [Google](https://google.com)"),
            "Visit Google"
        );

        // Test combined markdown
        assert_eq!(
            segmenter.clean_markdown("**准备** *材料* 和 `工具`"),
            "准备 材料 和 工具"
        );
    }

    #[test]
    fn test_strict_chinese_segmentation() {
        let mut segmenter = StreamSegmenter::new(10);

        // Test that long Chinese text gets cut at 10 characters even with comma
        let text = "在锅中留少量底油，放入豆角煸炒至颜色变深";

        // This should trigger segmentation at the comma after "底油"
        assert_eq!(segmenter.add_chunk("在锅中留少量底油"), None); // 8 chars, no output yet
        let result = segmenter.add_chunk("，放入豆角煸炒"); // Adding more text with comma
        assert_eq!(result, Some("在锅中留少量底油，".to_string())); // Should segment at comma

        // The rest should be in buffer
        assert_eq!(segmenter.flush(), Some("放入豆角煸炒".to_string()));
    }
}
