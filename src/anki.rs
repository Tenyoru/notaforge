use ankiconnect_rs::{AnkiClient, Deck, Model, models::FieldRef};
use anyhow::{Result, anyhow};

/// Finds a deck by name
///
/// If a deck named `"Default"` is requested and it doesn’t exist,
/// a new `"Default"` deck will be created automatically.
/// For all other names, an error is returned if the deck is not found.
pub fn find_deck(client: &AnkiClient, name: &str) -> Result<Deck> {
    Ok(client
        .decks()
        .get_all()?
        .into_iter()
        .find(|d| d.name() == name)
        .ok_or_else(|| anyhow!("Deck '{}' not found", name))?)
}

/// Find a model by name.
pub fn find_model(client: &AnkiClient, name: &str) -> Result<Model> {
    Ok(client
        .models()
        .get_all()?
        .into_iter()
        .find(|m| m.name() == name)
        .ok_or_else(|| anyhow!("Model '{}' not found", name))?)
}

/// Get a field from the model by name, or return an error if it doesn't exist.
#[inline(always)]
pub fn get_model_field<'a>(model: &'a Model, name: &str) -> Result<FieldRef<'a>> {
    model
        .field_ref(name)
        .ok_or_else(|| anyhow!("Missing '{}' field", name))
}
