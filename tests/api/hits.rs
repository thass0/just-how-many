use crate::helper::TestApp;
use uuid::Uuid;

#[tokio::test]
async fn get_number_of_hits_works() {
    const URL: &str = "https://example.com/";
    let test_app = TestApp::spawn().await;

    let n_hits = 127;

    sqlx::query!(
	r#"
INSERT INTO pages (page_id, hits, owner, url)
VALUES ($1, $2, $3, $4)"#,
	Uuid::new_v4(),
	n_hits,
	Uuid::new_v4(),
	URL,
    )
	.execute(&test_app.db)
	.await
	.expect("Failed to set hits");
    
    let response = test_app.get_hits(URL).await;
    assert!(response.status().is_success());
    let hits_received = response
	.json::<i32>()
	.await
	.expect("Failed to receive hits");
    assert_eq!(hits_received, n_hits);
}
