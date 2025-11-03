use std::collections::BTreeSet;

use anyhow::{Context, Result, anyhow};
use reqwest::Client;
use serde::Deserialize;

use crate::card_template::{ExampleSentence, VocabularyCard};

const DICTIONARY_ENDPOINT: &str = "https://api.dictionaryapi.dev/api/v2/entries/en/";
const DATAMUSE_ENDPOINT: &str = "https://api.datamuse.com/words";
const DEFAULT_TRANSLATE_BASE: &str = "https://lingva.ml/api/v1";

pub async fn build_vocabulary_card(
    client: &Client,
    term: &str,
    source_lang: &str,
    target_lang: &str,
    translate_base: Option<&str>,
) -> Result<VocabularyCard> {
    let dictionary = fetch_dictionary_entry(client, term)
        .await
        .unwrap_or_default();
    let mut synonyms_set: BTreeSet<String> = dictionary.synonyms.iter().cloned().collect();
    let datamuse_synonyms = fetch_datamuse_synonyms(client, term)
        .await
        .unwrap_or_default();
    synonyms_set.extend(datamuse_synonyms);
    let synonyms: Vec<String> = synonyms_set.into_iter().collect();

    let part_of_speech = dictionary.part_of_speech.unwrap_or_default();
    let pronunciation = dictionary.pronunciation.unwrap_or_default();

    let translation = translate_text(client, term, source_lang, target_lang, translate_base)
        .await
        .with_context(|| format!("failed to translate '{term}'"))?;

    let synonyms_joined = synonyms.join(", ");
    let translated_synonyms = if synonyms_joined.is_empty() {
        String::new()
    } else {
        let mut translated = Vec::with_capacity(synonyms.len());
        for synonym in &synonyms {
            match translate_text(client, synonym, source_lang, target_lang, translate_base).await {
                Ok(value) => translated.push(value),
                Err(_) => translated.push(synonym.clone()),
            }
        }
        translated.join(", ")
    };

    let definition_text = dictionary
        .definition
        .unwrap_or_else(|| format!("No definition found for {term}."));

    let translated_usage = translate_text(
        client,
        &definition_text,
        source_lang,
        target_lang,
        translate_base,
    )
    .await
    .unwrap_or(definition_text.clone());

    let example_sentence = dictionary
        .example
        .unwrap_or_else(|| format!("This sentence uses the word {term}."));

    let highlight = if example_sentence.contains(term) {
        term.to_string()
    } else {
        String::new()
    };

    Ok(VocabularyCard {
        term: term.to_string(),
        pronunciation,
        part_of_speech,
        example: ExampleSentence {
            sentence: example_sentence,
            highlight,
        },
        translation_heading: translation,
        translation_synonyms: translated_synonyms,
        translation_usage: translated_usage,
        extra_tags: vec![
            source_lang.to_string(),
            target_lang.to_string(),
            "auto-generated".to_string(),
        ],
    })
}

async fn fetch_dictionary_entry(client: &Client, term: &str) -> Result<DictionaryData> {
    let url = format!("{DICTIONARY_ENDPOINT}{term}");
    let entries: Vec<DictionaryEntry> = client
        .get(&url)
        .send()
        .await
        .context("Dictionary request failed")?
        .error_for_status()
        .context("Dictionary service returned error")?
        .json()
        .await
        .context("Dictionary response parsing failed")?;

    let entry = entries
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("No dictionary entry for '{term}'"))?;

    let pronunciation = entry
        .phonetic
        .clone()
        .or_else(|| entry.phonetics.iter().find_map(|p| p.text.clone()));

    let meaning = entry
        .meanings
        .into_iter()
        .find(|meaning| !meaning.definitions.is_empty())
        .ok_or_else(|| anyhow!("Dictionary missing definitions for '{term}'"))?;

    let definitions = meaning.definitions.clone();

    let definition = definitions
        .iter()
        .find_map(|def| (!def.definition.is_empty()).then(|| def.definition.clone()));

    let example = definitions.iter().find_map(|def| def.example.clone());

    let synonyms = collect_synonyms(&definitions, meaning.synonyms);

    Ok(DictionaryData {
        pronunciation,
        part_of_speech: meaning.part_of_speech,
        definition,
        example,
        synonyms,
    })
}

async fn fetch_datamuse_synonyms(client: &Client, term: &str) -> Result<Vec<String>> {
    let response: Vec<DatamuseEntry> = client
        .get(DATAMUSE_ENDPOINT)
        .query(&[("rel_syn", term), ("max", "5")])
        .send()
        .await
        .context("Datamuse request failed")?
        .error_for_status()
        .context("Datamuse returned error")?
        .json()
        .await
        .context("Datamuse response parsing failed")?;

    Ok(response.into_iter().map(|entry| entry.word).collect())
}

async fn translate_text(
    client: &Client,
    text: &str,
    source_lang: &str,
    target_lang: &str,
    translate_base: Option<&str>,
) -> Result<String> {
    if text.trim().is_empty() {
        return Ok(String::new());
    }

    #[derive(Deserialize)]
    struct LingvaResponse {
        translation: String,
    }

    let base = translate_base.unwrap_or(DEFAULT_TRANSLATE_BASE);
    let base = base.trim_end_matches('/');
    let url = format!(
        "{}/{}/{}/{}",
        base,
        source_lang,
        target_lang,
        urlencoding::encode(text)
    );

    let response: LingvaResponse = client
        .get(url)
        .send()
        .await
        .context("Lingva request failed")?
        .error_for_status()
        .context("Lingva returned error")?
        .json()
        .await
        .context("Lingva response parsing failed")?;

    Ok(response.translation)
}

fn collect_synonyms(definitions: &[Definition], base_synonyms: Vec<String>) -> Vec<String> {
    let mut set: BTreeSet<String> = base_synonyms.into_iter().collect();
    for definition in definitions {
        for synonym in &definition.synonyms {
            set.insert(synonym.clone());
        }
    }
    set.into_iter().collect()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DictionaryEntry {
    phonetic: Option<String>,
    #[serde(default)]
    phonetics: Vec<Phonetic>,
    #[serde(default)]
    meanings: Vec<Meaning>,
}

#[derive(Deserialize)]
struct Phonetic {
    #[serde(default)]
    text: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Meaning {
    #[serde(default)]
    part_of_speech: Option<String>,
    #[serde(default)]
    definitions: Vec<Definition>,
    #[serde(default)]
    synonyms: Vec<String>,
}

#[derive(Clone, Deserialize)]
struct Definition {
    definition: String,
    #[serde(default)]
    example: Option<String>,
    #[serde(default)]
    synonyms: Vec<String>,
}

#[derive(Deserialize)]
struct DatamuseEntry {
    word: String,
}

#[derive(Default)]
struct DictionaryData {
    pronunciation: Option<String>,
    part_of_speech: Option<String>,
    definition: Option<String>,
    example: Option<String>,
    synonyms: Vec<String>,
}
