use std::sync::{Arc, Mutex};

use crate::model::{ApiResponse, Definition, Meaning, WordEntry};
use anyhow::Result;
use log::info;
use lru::LruCache;
use sqlx::{postgres::PgConnectOptions, ConnectOptions, PgPool, Pool, Postgres};
use url::Url;

#[derive(Clone)]
pub(crate) struct Repository {
    pool: Pool<Postgres>,
    client: reqwest::Client,
    dictionary_api: Url,
    words_cache: Arc<Mutex<LruCache<String, Vec<WordEntry>>>>,
}

struct DbWord {
    word: String,
}

struct DbWordEntry {
    id: i32,
    word: String,
}

struct DbSourceUrl {
    id: i32,
    word_entry_id: i32,
    url: String,
}

struct DbMeaning {
    id: i32,
    word_entry_id: i32,
    part_of_speech: String,
}

struct DbDefinition {
    id: i32,
    meaning_id: i32,
    definition: String,
    example: Option<String>,
}

struct DbSynonym {
    id: i32,
    meaning_id: i32,
    synonym: String,
}

struct DbAntonym {
    id: i32,
    meaning_id: i32,
    antonym: String,
}

impl Repository {
    pub(crate) async fn initialize(database_url: Url) -> Result<Self> {
        info!("Initializing repository with url: {database_url}");
        let options = PgConnectOptions::from_url(&database_url)?;

        info!("Trying to connect to database...");
        let pool = PgPool::connect_with(options).await?;
        info!("Connection successfull");

        sqlx::migrate!().run(&pool).await?;
        info!("Migrations applied successfully");

        let client = reqwest::Client::builder()
            .user_agent("Dictionary webapp.")
            .build()
            .unwrap();

        let dictionary_api =
            Url::parse("https://api.dictionaryapi.dev/api/v2/entries/en/").unwrap();

        let words_cache = Arc::new(Mutex::new(LruCache::new(
            std::num::NonZero::new(100).unwrap(),
        )));

        Ok(Self {
            pool,
            client,
            dictionary_api,
            words_cache,
        })
    }

    pub(crate) async fn request_word_definitions(&self, word: &str) -> Result<ApiResponse> {
        info!("Request definition for word: '{word}'");
        let word_url = self.dictionary_api.join(word)?;
        info!("API url: '{word_url}'");
        let request = self.client.get(word_url);

        info!("Sending request...");
        let response = request.send().await?;

        info!("Response received for word: '{word}'");
        let word_definitions = response.json::<ApiResponse>().await?;

        Ok(word_definitions)
    }

    pub(crate) async fn add_word_entries(
        &self,
        word: &str,
        word_entries: Vec<WordEntry>,
    ) -> Result<()> {
        let mut transaction = self.pool.begin().await?;

        for word_entry in &word_entries {
            let word_entry_id: i32 = sqlx::query!(
                r#"
                insert into word_entries (word)
                values ($1)
                returning id
                "#,
                word_entry.word
            )
            .fetch_one(&mut *transaction)
            .await?
            .id;

            for url in &word_entry.source_urls {
                sqlx::query!(
                    r#"
                    insert into source_urls (word_entry_id, url)
                    values ($1, $2)
                    "#,
                    word_entry_id,
                    url
                )
                .execute(&mut *transaction)
                .await?;
            }

            for meaning in &word_entry.meanings {
                let meaning_id: i32 = sqlx::query!(
                    r#"
                    insert into meanings (word_entry_id, part_of_speech)
                    values ($1, $2)
                    returning id
                    "#,
                    word_entry_id,
                    meaning.part_of_speech
                )
                .fetch_one(&mut *transaction)
                .await?
                .id;

                for definition in &meaning.definitions {
                    sqlx::query!(
                        r#"
                        insert into definitions (meaning_id, definition, example)
                        values ($1, $2, $3)
                        "#,
                        meaning_id,
                        definition.definition,
                        definition.example
                    )
                    .execute(&mut *transaction)
                    .await?;
                }

                for synonym in &meaning.synonyms {
                    sqlx::query!(
                        r#"
                        insert into synonyms (meaning_id, synonym)
                        values ($1, $2)
                        "#,
                        meaning_id,
                        synonym
                    )
                    .execute(&mut *transaction)
                    .await?;
                }

                for antonym in &meaning.antonyms {
                    sqlx::query!(
                        r#"
                        insert into antonyms (meaning_id, antonym)
                        values ($1, $2)
                        "#,
                        meaning_id,
                        antonym
                    )
                    .execute(&mut *transaction)
                    .await?;
                }
            }
        }

        sqlx::query!(
            r#"
            insert into words (word)
            values ($1)
            "#,
            word
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;

        {
            let mut cache_guard = self.words_cache.lock().unwrap();

            cache_guard.put(word.to_owned(), word_entries);
        }

        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    pub(crate) async fn get_word_definitions(&self, word: &str) -> Result<Option<Vec<WordEntry>>> {
        {
            let mut cache_guard = self.words_cache.lock().unwrap();

            if let Some(word_entries) = cache_guard.get(word) {
                return Ok(Some(word_entries.clone()));
            }
        }

        let mut transaction = self.pool.begin().await?;

        let query = sqlx::query_as!(
            DbWordEntry,
            r#"
            select id as "id!", word as "word!"
            from word_entries
            where word = $1
            "#,
            word
        );

        let db_word_entries = query.fetch_all(&mut *transaction).await?;

        if db_word_entries.is_empty() {
            return Ok(None);
        }

        let mut word_entries = Vec::new();

        for db_word_entry in db_word_entries {
            let mut word_entry = WordEntry {
                word: word.to_owned(),
                meanings: Vec::new(),
                source_urls: Vec::new(),
            };

            let query = sqlx::query_as!(
                DbSourceUrl,
                r#"
                select id as "id!", word_entry_id as "word_entry_id!", url as "url!"
                from source_urls
                where word_entry_id = $1
                "#,
                db_word_entry.id
            );

            let db_source_urls = query.fetch_all(&mut *transaction).await?;

            word_entry.source_urls.extend(
                db_source_urls
                    .into_iter()
                    .map(|DbSourceUrl { url, .. }| url),
            );

            let query = sqlx::query_as!(
                DbMeaning,
                r#"
                select id as "id!", word_entry_id as "word_entry_id!", part_of_speech as "part_of_speech!"
                from meanings
                where word_entry_id = $1
                "#,
                db_word_entry.id
            );

            let db_meanings = query.fetch_all(&mut *transaction).await?;

            for db_meaning in db_meanings {
                let mut meaning = Meaning {
                    part_of_speech: db_meaning.part_of_speech,
                    definitions: Vec::new(),
                    synonyms: Vec::new(),
                    antonyms: Vec::new(),
                };

                let query = sqlx::query_as!(
                    DbDefinition,
                    r#"
                    select id as "id!", meaning_id as "meaning_id!", definition as "definition!", example
                    from definitions
                    where meaning_id = $1
                    "#,
                    db_meaning.id
                );

                let db_definitions = query.fetch_all(&mut *transaction).await?;
                meaning.definitions.extend(db_definitions.into_iter().map(
                    |DbDefinition {
                         definition,
                         example,
                         ..
                     }| {
                        Definition {
                            definition,
                            example,
                        }
                    },
                ));

                let query = sqlx::query_as!(
                    DbSynonym,
                    r#"
                    select id as "id!", meaning_id as "meaning_id!", synonym as "synonym!"
                    from synonyms
                    where meaning_id = $1
                    "#,
                    db_meaning.id
                );

                let db_synonyms = query.fetch_all(&mut *transaction).await?;
                meaning.synonyms.extend(
                    db_synonyms
                        .into_iter()
                        .map(|DbSynonym { synonym, .. }| synonym),
                );

                let query = sqlx::query_as!(
                    DbAntonym,
                    r#"
                    select id as "id!", meaning_id as "meaning_id!", antonym as "antonym!"
                    from antonyms
                    where meaning_id = $1
                    "#,
                    db_meaning.id
                );

                let db_antonyms = query.fetch_all(&mut *transaction).await?;
                meaning.antonyms.extend(
                    db_antonyms
                        .into_iter()
                        .map(|DbAntonym { antonym, .. }| antonym),
                );

                word_entry.meanings.push(meaning);
            }
            word_entries.push(word_entry);
        }

        transaction.commit().await?;

        {
            let mut cache_guard = self.words_cache.lock().unwrap();

            cache_guard.put(word.to_owned(), word_entries.clone());
        }

        Ok(Some(word_entries))
    }

    pub(crate) async fn get_10_random_words(&self) -> Result<Vec<String>> {
        let mut transaction = self.pool.begin().await?;

        let query = sqlx::query_as!(
            DbWord,
            r#"
            select word
            from words
            order by random()
            limit 10
            "#
        );

        let words = query.fetch_all(&mut *transaction).await?;
        let words = words.into_iter().map(|DbWord { word }| word).collect();

        transaction.commit().await?;

        Ok(words)
    }

    pub(crate) async fn get_all_words(&self) -> Result<Vec<String>> {
        let mut transaction = self.pool.begin().await?;

        let query = sqlx::query_as!(
            DbWord,
            r#"
            select word
            from words
            "#
        );

        let words = query.fetch_all(&mut *transaction).await?;
        let words = words.into_iter().map(|DbWord { word }| word).collect();

        transaction.commit().await?;

        Ok(words)
    }
}
