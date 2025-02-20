use crate::model::{Word, WordDefinitions, WordEntry};
use anyhow::Result;
use sqlx::{postgres::PgConnectOptions, ConnectOptions, PgPool, Pool, Postgres};
use url::Url;

#[derive(Clone)]
pub(crate) struct Repository {
    pool: Pool<Postgres>,
    client: reqwest::Client,
    dictionary_api: Url,
}

impl Repository {
    pub(crate) async fn initialize(database_url: Url) -> Result<Self> {
        let options = PgConnectOptions::from_url(&database_url)?;

        let pool = PgPool::connect_with(options).await?;

        // sqlx::migrate!().run(&pool).await?;

        let client = reqwest::Client::builder()
            .user_agent("Dictionary webapp.")
            .build()
            .unwrap();

        let dictionary_api =
            Url::parse("https://api.dictionaryapi.dev/api/v2/entries/en/").unwrap();

        Ok(Self {
            pool,
            client,
            dictionary_api,
        })
    }

    pub(crate) async fn get_word_definitions(&self, word: &Word) -> Result<WordDefinitions> {
        let word_url = self.dictionary_api.join(word.as_str())?;
        let request = self.client.get(word_url);

        let response = request.send().await?;

        let word_definitions = response.json::<WordDefinitions>().await?;

        Ok(word_definitions)
    }

    pub(crate) async fn add_word_definitions(&self, word_definition: Vec<WordEntry>) -> Result<()> {
        let mut transaction = self.pool.begin().await?;

        for word_entry in word_definition {
            let word_entry_id: i32 = sqlx::query!(
                r#"
                insert into word_entry (word, phonetic)
                values ($1, $2)
                returning id
                "#,
                word_entry.word,
                word_entry.phonetic
            )
            .fetch_one(&mut *transaction)
            .await?
            .id;

            for url in word_entry.source_urls {
                sqlx::query!(
                    r#"
                    insert into source_url (word_entry_id, url)
                    values ($1, $2)
                    "#,
                    word_entry_id,
                    url
                )
                .execute(&mut *transaction)
                .await?;
            }

            for phonetic in word_entry.phonetics {
                sqlx::query!(
                    r#"
                    insert into phonetic (word_entry_id, text, audio, source_url)
                    values ($1, $2, $3, $4)
                    "#,
                    word_entry_id,
                    phonetic.text,
                    phonetic.audio,
                    phonetic.source_url
                )
                .execute(&mut *transaction)
                .await?;
            }

            for meaning in word_entry.meanings {
                let meaning_id: i32 = sqlx::query!(
                    r#"
                    insert into meaning (word_entry_id, part_of_speech)
                    values ($1, $2)
                    returning id
                    "#,
                    word_entry_id,
                    meaning.part_of_speech
                )
                .fetch_one(&mut *transaction)
                .await?
                .id;

                for definition in meaning.definitions {
                    sqlx::query!(
                        r#"
                        insert into definition (meaning_id, definition, example)
                        values ($1, $2, $3)
                        "#,
                        meaning_id,
                        definition.definition,
                        definition.example
                    )
                    .execute(&mut *transaction)
                    .await?;
                }

                for synonym in meaning.synonyms {
                    sqlx::query!(
                        r#"
                        insert into synonym (meaning_id, synonym)
                        values ($1, $2)
                        "#,
                        meaning_id,
                        synonym
                    )
                    .execute(&mut *transaction)
                    .await?;
                }

                for antonym in meaning.antonyms {
                    sqlx::query!(
                        r#"
                        insert into antonym (meaning_id, antonym)
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

        transaction.commit().await?;

        Ok(())
    }
}
