use directories;
use include_dir::include_dir;
use std::{env, path::PathBuf};
use surrealdb_migrations::MigrationRunner;

use surrealdb::{
  engine::any::Any, opt::auth::Root, sql::Thing, RecordId, Surreal,
};

pub struct Client {
  private: Surreal<Any>,
  public: Surreal<Any>,
}

pub async fn connect(embed: bool) -> anyhow::Result<Client> {
  Ok(Client::new(embed).await?)
}

#[derive(Debug, Clone)]
pub struct Chat {
  pub id: String,
  pub timestamp: chrono::DateTime<chrono::Utc>,
  pub last_interaction: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
pub struct Message {
  pub id: String,
  pub chat: String,
  pub timestamp: chrono::DateTime<chrono::Utc>,
  pub content: String,
  pub sender: String,
}

#[derive(Debug, Clone)]
pub struct FullTextSearch<T: Clone> {
  pub record: T,
  pub highlights: String,
  pub score: f32,
}

impl Client {
  pub async fn insert_chat(&self) -> anyhow::Result<Chat> {
    #[derive(serde::Serialize)]
    struct InChat {
      timestamp: Option<chrono::DateTime<chrono::Utc>>,
      last_interaction: Option<chrono::DateTime<chrono::Utc>>,
    }

    #[derive(serde::Deserialize)]
    struct OutChat {
      id: Thing,
      timestamp: chrono::DateTime<chrono::Utc>,
      last_interaction: Option<chrono::DateTime<chrono::Utc>>,
    }

    let chat = self
      .public
      .create::<Option<OutChat>>("chat")
      .content(InChat {
        timestamp: None,
        last_interaction: None,
      })
      .await?
      .ok_or_else(|| anyhow::anyhow!("Database returned none"))?;

    Ok(Chat {
      id: chat.id.id.to_raw(),
      timestamp: chat.timestamp,
      last_interaction: chat.last_interaction,
    })
  }

  pub async fn insert_message(
    &self,
    chat: String,
    sender: String,
    content: String,
  ) -> anyhow::Result<Message> {
    #[derive(serde::Serialize)]
    struct InMessage {
      timestamp: Option<chrono::DateTime<chrono::Utc>>,
      sender: String,
      content: String,
    }

    #[derive(serde::Deserialize)]
    struct OutMessage {
      pub timestamp: chrono::DateTime<chrono::Utc>,
      pub content: String,
      pub sender: String,
    }

    #[derive(serde::Deserialize)]
    struct OutPostedIn {
      #[serde(rename = "in")]
      message: RecordId,
      #[serde(rename = "out")]
      chat: RecordId,
    }

    let mut response = self
      .public
      .query(
        r#"
          BEGIN;
          LET $inserted = (INSERT INTO message $message)[0];
          $inserted;
          RELATE ($inserted.id)->posted_in->$chat;
          COMMIT;
        "#,
      )
      .bind((
        "message",
        InMessage {
          timestamp: None,
          sender,
          content,
        },
      ))
      .bind(("chat", RecordId::from(("chat", chat))))
      .await?;

    let message = response
      .take::<Vec<OutMessage>>(1)?
      .into_iter()
      .nth(0)
      .ok_or_else(|| anyhow::anyhow!("Database returned none"))?;
    let posted_in = response
      .take::<Vec<OutPostedIn>>(2)?
      .into_iter()
      .nth(0)
      .ok_or_else(|| anyhow::anyhow!("Database returned none"))?;

    Ok(Message {
      id: posted_in.message.key().to_string(),
      chat: posted_in.chat.key().to_string(),
      timestamp: message.timestamp,
      content: message.content,
      sender: message.sender,
    })
  }

  pub async fn search_messages(
    &self,
    content: &str,
  ) -> anyhow::Result<Vec<FullTextSearch<Message>>> {
    #[derive(serde::Deserialize)]
    struct OutMessage {
      id: Thing,
      chat: Thing,
      timestamp: chrono::DateTime<chrono::Utc>,
      content: String,
      sender: String,
      highlights: String,
      score: f32,
    }

    let query = r#"
      SELECT
        *,
        (->posted_in->chat.id)[0] AS chat,
        search::highlight('<b>', '</b>', 1) AS highlights,
        search::score(1) AS score
      FROM message
      WHERE content @1@ $content;
    "#;

    let messages = self
      .public
      .query(query)
      .bind(("content", content.to_owned()))
      .await?
      .take::<Vec<OutMessage>>(0)?;

    return Ok(
      messages
        .into_iter()
        .map(|message| FullTextSearch {
          record: Message {
            id: message.id.id.to_raw(),
            chat: message.chat.id.to_raw(),
            timestamp: message.timestamp,
            content: message.content,
            sender: message.sender,
          },
          highlights: message.highlights,
          score: message.score,
        })
        .collect::<Vec<_>>(),
    );
  }

  pub async fn migrate(&self) -> anyhow::Result<()> {
    if let Err(err) = MigrationRunner::new(&self.private)
      .load_files(&include_dir!("$CARGO_MANIFEST_DIR/migrations/private"))
      .up()
      .await
    {
      let err = err.to_string();
      return Err(anyhow::anyhow!(format!("Private migration failed: {err}")));
    }

    if let Err(err) = MigrationRunner::new(&self.public)
      .load_files(&include_dir!("$CARGO_MANIFEST_DIR/migrations/public"))
      .up()
      .await
    {
      let err = err.to_string();
      return Err(anyhow::anyhow!(format!("Public migration failed: {err}")));
    }

    Ok(())
  }

  pub(crate) async fn new(embed: bool) -> anyhow::Result<Self> {
    let address = if embed {
      let project_dirs =
        directories::ProjectDirs::from("xyz", "haras-unicorn", "double-star")
          .ok_or_else(|| anyhow::anyhow!("no project directories"))?;
      PathBuf::from(project_dirs.project_path())
        .join("data")
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("failed creating database"))?
        .to_owned()
    } else {
      let host = env::var("DOUBLE_STAR_DB_HOST")?;
      let port = env::var("DOUBLE_STAR_DB_PORT")?;
      format!("ws://{host}:{port}")
    };

    let user = env::var("DOUBLE_STAR_DB_USER")?;
    let pass = env::var("DOUBLE_STAR_DB_PASS")?;

    let private = surrealdb::engine::any::connect(address.clone()).await?;
    private
      .signin(Root {
        username: user.as_str(),
        password: pass.as_str(),
      })
      .await?;
    private.use_ns("double_star").await?;
    private.use_db("private").await?;

    let public = surrealdb::engine::any::connect(address).await?;
    public
      .signin(Root {
        username: user.as_str(),
        password: pass.as_str(),
      })
      .await?;
    public.use_ns("double_star").await?;
    public.use_db("public").await?;

    Ok(Self { private, public })
  }
}
