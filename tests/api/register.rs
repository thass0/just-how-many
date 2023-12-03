use crate::helper::TestApp;
use serde::Deserialize;
use uuid::Uuid;

#[tokio::test]
async fn register_400s_on_missing_data() {
    let test_app = TestApp::spawn().await;
    let response = test_app.post_register("url=").await;
    assert_eq!(400, response.status().as_u16());
}

#[tokio::test]
async fn register_400s_on_malformed_data() {
    let test_app = TestApp::spawn().await;
    let response = test_app.post_register("url=This is not a valid URL.").await;
    assert_eq!(400, response.status().as_u16());
}

#[derive(Deserialize)]
struct PageId {
    id: Uuid,
}

#[tokio::test]
async fn register_creates_and_returns_a_page() {
    let test_app = TestApp::spawn().await;
    let url = "https://example.com";
    let response = test_app.post_register(&url).await;
    assert_eq!(200, response.status().as_u16());
    let page_id = response.json::<PageId>().await.unwrap().id;
    let page = sqlx::query!(
	r#"
SELECT *
FROM pages
WHERE page_id = $1
"#,
	page_id)
	.fetch_one(&test_app.db)
	.await
	.expect("Failed to retrieve page");
    assert_eq!(&page.url, &url);
}
