pub struct CardFields {
    pub front: String,
    pub back: String,
    pub tags: Vec<String>,
}

pub trait CardTemplate {
    fn render(&self) -> CardFields;
}

#[derive(Default)]
pub struct ExampleSentence {
    pub sentence: String,
    pub highlight: String,
}

impl ExampleSentence {
    fn render(&self) -> String {
        if self.highlight.is_empty() {
            return self.sentence.clone();
        }

        let highlight_html = format!(
            "<span style=\"text-decoration:underline; color:red;\">{}</span>",
            self.highlight
        );

        if self.sentence.contains(&self.highlight) {
            self.sentence.replacen(&self.highlight, &highlight_html, 1)
        } else {
            self.sentence.clone()
        }
    }
}

pub struct VocabularyCard {
    pub term: String,
    pub pronunciation: String,
    pub part_of_speech: String,
    pub example: ExampleSentence,
    pub translation_heading: String,
    pub translation_synonyms: String,
    pub translation_usage: String,
    pub extra_tags: Vec<String>,
}

impl CardTemplate for VocabularyCard {
    fn render(&self) -> CardFields {
        let part_display = if self.part_of_speech.is_empty() {
            String::new()
        } else {
            format!(" · {}", self.part_of_speech)
        };

        let front = format!(
            concat!(
                "<b style=\"font-size:1.4em;\">{term}</b>",
                "<br><span style=\"color:#888;\">{pronunciation}{part_display}</span>",
                "<br><br><i>{example}</i>",
            ),
            term = self.term,
            pronunciation = self.pronunciation,
            part_display = part_display,
            example = self.example.render(),
        );

        let back = format!(
            concat!(
                "<div style=\"margin-bottom:0.2em;\">",
                "<b style=\"font-size:1.2em;\">{heading}</b>",
                "</div>",
                "<div style=\"margin-bottom:0.8em; color:#5e84c1;\">",
                "{synonyms}</div>",
                "<div style=\"margin-bottom:1em; font-size:0.95em; ",
                "line-height:1.5em; color:#ccc;\">{usage}</div>",
            ),
            heading = self.translation_heading,
            synonyms = self.translation_synonyms,
            usage = self.translation_usage,
        );

        let mut tags = Vec::new();
        if !self.part_of_speech.is_empty() {
            tags.push(self.part_of_speech.clone());
        }
        tags.extend(self.extra_tags.iter().filter(|t| !t.is_empty()).cloned());

        CardFields { front, back, tags }
    }
}

pub struct SimpleCard {
    pub front: String,
    pub back: String,
    pub tags: Vec<String>,
}

impl CardTemplate for SimpleCard {
    fn render(&self) -> CardFields {
        CardFields {
            front: self.front.clone(),
            back: self.back.clone(),
            tags: self.tags.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_example_sentence_with_highlight() {
        let example = ExampleSentence {
            sentence: "I was taken aback by her sudden outburst.".to_string(),
            highlight: "taken aback".to_string(),
        };

        let rendered = example.render();
        assert!(rendered.contains("taken aback"));
        assert!(rendered.contains("text-decoration:underline"));
    }

    #[test]
    fn renders_vocabulary_card_fields() {
        let card = VocabularyCard {
            term: "aback".to_string(),
            pronunciation: "/əˈbæk/".to_string(),
            part_of_speech: "adverb".to_string(),
            example: ExampleSentence {
                sentence: "I was taken aback by her sudden outburst.".to_string(),
                highlight: "taken aback".to_string(),
            },
            translation_heading: "застигнутый врасплох".to_string(),
            translation_synonyms: "удивлённый".to_string(),
            translation_usage: "Используется при внезапном удивлении.".to_string(),
            extra_tags: vec!["english".to_string(), "emotion".to_string()],
        };

        let fields = card.render();
        assert!(fields.front.contains("aback"));
        assert!(fields.back.contains("застигнутый"));
        assert!(fields.tags.contains(&"adverb".to_string()));
        assert!(fields.tags.contains(&"english".to_string()));
    }

    #[test]
    fn renders_simple_card() {
        let card = SimpleCard {
            front: "Front".to_string(),
            back: "Back".to_string(),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };

        let fields = card.render();
        assert_eq!(fields.front, "Front");
        assert_eq!(fields.back, "Back");
        assert_eq!(fields.tags.len(), 2);
    }
}
