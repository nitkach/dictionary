use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct PostWord {
    #[serde(flatten)]
    word: Word,
}

impl PostWord {
    pub(crate) fn into_word(self) -> Word {
        self.word
    }
}

#[derive(Deserialize)]
pub(crate) struct Word(String);

impl Word {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub(crate) enum WordDefinitions {
    Exists(Vec<WordEntry>),
    Missing {
        title: String,
        message: String,
        resolution: String,
    },
}

#[derive(Deserialize, Debug)]
pub(crate) struct WordEntry {
    pub(crate) word: String,
    pub(crate) phonetic: String,
    pub(crate) phonetics: Vec<Phonetic>,
    pub(crate) meanings: Vec<Meaning>,
    #[serde(rename = "sourceUrls")]
    pub(crate) source_urls: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct Phonetic {
    pub(crate) text: String,
    pub(crate) audio: String,
    #[serde(rename = "sourceUrl")]
    pub(crate) source_url: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct Meaning {
    #[serde(rename = "partOfSpeech")]
    pub(crate) part_of_speech: String, // (noun)(adjective)(verb)(preposition)(adverb)(pronoun)(conjunction)(interjection)
    pub(crate) definitions: Vec<Definition>,
    pub(crate) synonyms: Vec<String>,
    pub(crate) antonyms: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct Definition {
    pub(crate) definition: String,
    pub(crate) example: String,
}

#[derive(Deserialize, Debug)]
pub(crate) enum PartOfSpeech {
    Noun,
    Pronoun,
    Adjective,
    Adverb,
    Verb,
    Preposition,
    Conjunction,
    Interjection,
}
