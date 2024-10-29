mod common;

#[tokio::test]
async fn test_message_search() -> anyhow::Result<()> {
  let client = common::setup().await?;

  let content_search = "some";
  let content_other = "content";
  let content = format!("{content_search} {content_other}");

  let chat = client.insert_chat().await?;
  let _ = client
    .insert_message(chat.id, "sender".to_string(), content)
    .await?;

  let result = client
    .search_messages(content_search)
    .await?
    .into_iter()
    .map(|result| result.highlights)
    .collect::<Vec<_>>();

  assert_eq!(
    result,
    vec![format!("<b>{content_search}</b> {content_other}")]
  );

  Ok(())
}
