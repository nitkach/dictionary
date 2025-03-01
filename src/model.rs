use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub(crate) struct AddWordForm {
    pub(crate) word: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub(crate) enum ApiResponse {
    Success(Vec<WordEntry>),
    Missing(MissingResponse),
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct MissingResponse {
    pub(crate) title: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct WordEntry {
    pub(crate) word: String,
    pub(crate) meanings: Vec<Meaning>,
    #[serde(rename = "sourceUrls")]
    pub(crate) source_urls: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct Meaning {
    #[serde(rename = "partOfSpeech")]
    pub(crate) part_of_speech: String,
    pub(crate) definitions: Vec<Definition>,
    pub(crate) synonyms: Vec<String>,
    pub(crate) antonyms: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct Definition {
    pub(crate) definition: String,
    pub(crate) example: Option<String>,
}
