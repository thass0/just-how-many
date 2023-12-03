use crate::helper::TestApp;
use uuid::Uuid;

#[tokio::test]
async fn register_400s_on_missing_data() {
    let test_app = TestApp::spawn().await;
    let response = test_app.post_register("url=").await;

    assert_eq!(400, response.status().as_u16());
    let bytes = response.bytes().await.unwrap();
    assert!(bytes.starts_with(
	"Parse error: invalid value: string \"\"".as_bytes()));
}

#[tokio::test]
async fn register_400s_on_malformed_data() {
    let test_app = TestApp::spawn().await;
    let response = test_app.post_register("url=This is not a valid URL.").await;

    assert_eq!(400, response.status().as_u16());
    let bytes = response.bytes().await.unwrap();
    assert!(bytes.starts_with(
	"Parse error: invalid value: string \"This is not a valid URL.\"".as_bytes()));
}

#[tokio::test]
async fn register_creates_and_returns_a_page() {
    const URL: &str = "https://example.com/";
    let test_app = TestApp::spawn().await;
    let response = test_app.post_register(
	&format!("url={URL}")).await;

    assert_eq!(200, response.status().as_u16());

    let page_id = response.json::<Uuid>().await.unwrap();
    let page = sqlx::query!(
	r#"
SELECT *
FROM pages
WHERE page_id = $1
"#,
	page_id)
	.fetch_one(&test_app.db)
	.await
	.unwrap();

    assert_eq!(
	&page.url,
	URL,
	"Stored URL should be the same as the transmitted URL"
    );
    assert_eq!(
	&page.page_id,
	&page_id,
	"Received page_id should be the same as the stored page_id"
    );
}
