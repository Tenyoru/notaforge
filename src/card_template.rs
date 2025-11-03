pub struct CardFields {
    pub front: String,
    pub back: String,
    pub tags: Vec<String>,
}

pub trait CardTemplate {
    fn render(&self) -> CardFields;
}

pub struct ExampleSentence<'a> {
    pub sentence: &'a str,
    pub highlight: &'a str,
}

impl<'a> ExampleSentence<'a> {
    fn render(&self) -> String {
        if self.highlight.is_empty() {
            return self.sentence.to_owned();
        }

        let highlight_html = format!(
            "<span style=\"text-decoration:underline; color:red;\">{}</span>",
            self.highlight
        );

        if self.sentence.contains(self.highlight) {
            self.sentence.replacen(self.highlight, &highlight_html, 1)
        } else {
            self.sentence.to_owned()
        }
    }
}

pub struct VocabularyCard<'a> {
    pub term: &'a str,
    pub pronunciation: &'a str,
    pub part_of_speech: &'a str,
    pub example: ExampleSentence<'a>,
    pub translation_heading: &'a str,
    pub translation_synonyms: &'a str,
    pub translation_usage: &'a str,
    pub extra_tags: &'a [&'a str],
}

impl<'a> CardTemplate for VocabularyCard<'a> {
    fn render(&self) -> CardFields {
        let front = format!(
            concat!(
                "<b style=\"font-size:1.4em;\">{term}</b>",
                "<br><span style=\"color:#888;\">{pronunciation} · {part_of_speech}</span>",
                "<br><br><i>{example}</i>",
            ),
            term = self.term,
            pronunciation = self.pronunciation,
            part_of_speech = self.part_of_speech,
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

        CardFields {
            front,
            back,
            tags: std::iter::once(self.part_of_speech.to_owned())
                .chain(self.extra_tags.iter().map(|t| t.to_string()))
                .collect(),
        }
    }
}

pub struct SimpleCard<'a> {
    pub front: &'a str,
    pub back: &'a str,
    pub tags: &'a [&'a str],
}

impl<'a> CardTemplate for SimpleCard<'a> {
    fn render(&self) -> CardFields {
        CardFields {
            front: self.front.to_owned(),
            back: self.back.to_owned(),
            tags: self.tags.iter().map(|tag| tag.to_string()).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_example_sentence_with_highlight() {
        let example = ExampleSentence {
            sentence: "I was taken aback by her sudden outburst.",
            highlight: "taken aback",
        };

        let rendered = example.render();
        assert!(rendered.contains("taken aback"));
        assert!(rendered.contains("text-decoration:underline"));
    }

    #[test]
    fn renders_vocabulary_card_fields() {
        let card = VocabularyCard {
            term: "aback",
            pronunciation: "/əˈbæk/",
            part_of_speech: "adverb",
            example: ExampleSentence {
                sentence: "I was taken aback by her sudden outburst.",
                highlight: "taken aback",
            },
            translation_heading: "застигнутый врасплох",
            translation_synonyms: "удивлённый",
            translation_usage: "Используется при внезапном удивлении.",
            extra_tags: &["english", "emotion"],
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
            front: "Front",
            back: "Back",
            tags: &["tag1", "tag2"],
        };

        let fields = card.render();
        assert_eq!(fields.front, "Front");
        assert_eq!(fields.back, "Back");
        assert_eq!(fields.tags.len(), 2);
    }
}
