mod anki;
use anki::*;
use ankiconnect_rs::{AnkiClient, MediaSource, NoteBuilder};
use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the Anki deck to use
    #[arg(short, long)]
    deck: String,

    /// Name of the Anki model to use (default: Basic)
    #[arg(short, long, default_value = "Basic")]
    model: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = AnkiClient::new();

    let deck = find_deck(&client, &args.deck)?;
    let model = find_model(&client, &args.model)?;

    let front_field = get_model_field(&model, "Front")?;
    let back_field = get_model_field(&model, "Back")?;

    let note = NoteBuilder::new(model.clone())
        .with_field(front_field, "¿Dónde está la biblioteca?")
        .with_field(back_field, "Where is the library?")
        .with_tag("spanish-vocab")
        .with_image(
            front_field,
            MediaSource::Url(
                "https://cdn.pixabay.com/photo/2023/08/18/15/02/dog-8198719_640.jpg".to_string(),
            ),
            "test_dog.jpg",
        )
        .build()?;

    // Add the note to the first deck
    let note_id = client.cards().add_note(&deck, note, false, None)?;
    println!("Added note with ID: {}", note_id.value());
    Ok(())
}
