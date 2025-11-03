mod anki;
mod card_template;
use anki::*;
use ankiconnect_rs::{AnkiClient, MediaSource, NoteBuilder};
use anyhow::Result;
use card_template::{CardTemplate, ExampleSentence, SimpleCard, VocabularyCard};
use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the Anki deck to use
    #[arg(short, long)]
    deck: String,

    /// Name of the Anki model to use (default: Basic)
    #[arg(short, long, default_value = "Basic")]
    model: String,

    /// Card template to use when generating fields
    #[arg(short, long, value_enum, default_value_t = TemplateKind::Vocabulary)]
    template: TemplateKind,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum TemplateKind {
    Vocabulary,
    Simple,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = AnkiClient::new();

    let deck = find_deck(&client, &args.deck)?;
    let model = find_model(&client, &args.model)?;

    let front_field = get_model_field(&model, "Front")?;
    let back_field = get_model_field(&model, "Back")?;

    let fields = match args.template {
        TemplateKind::Vocabulary => VocabularyCard {
            term: "aback",
            pronunciation: "/əˈbæk/",
            part_of_speech: "adverb",
            example: ExampleSentence {
                sentence: "I was taken aback by her sudden outburst.",
                highlight: "taken aback",
            },
            translation_heading: "застигнутый врасплох",
            translation_synonyms: "удивлённый, ошеломлённый, не ожидавший",
            translation_usage:
                "Используется, когда кто-то сильно удивлён или сбит с толку неожиданным событием.",
            extra_tags: &["english", "vocab"],
        }
        .render(),
        TemplateKind::Simple => SimpleCard {
            front: "<b>accident</b>",
            back: "<b>случайность</b>",
            tags: &["noun"],
        }
        .render(),
    };

    let mut builder = NoteBuilder::new(model.clone())
        .with_field_raw(front_field, &fields.front)
        .with_field_raw(back_field, &fields.back);

    for tag in &fields.tags {
        builder = builder.with_tag(tag);
    }

    builder = if matches!(args.template, TemplateKind::Vocabulary) {
        builder.with_image(
            front_field,
            MediaSource::Url(
                "https://cdn.pixabay.com/photo/2023/08/18/15/02/dog-8198719_640.jpg".to_string(),
            ),
            "test_dog.jpg",
        )
    } else {
        builder
    };

    let note = builder.build()?;

    // Add the note to the first deck
    let note_id = client.cards().add_note(&deck, note, false, None)?;
    println!("Added note with ID: {}", note_id.value());
    Ok(())
}
