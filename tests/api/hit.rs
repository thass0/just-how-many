use crate::helper::TestApp;
use rand::Rng;

#[tokio::test]
async fn hit_increment_works() {
    let test_app = TestApp::spawn().await;
    let page_id = test_app.insert_page().await;

    let n_hits = rand::thread_rng().gen_range(0..10);
    for _ in 0..n_hits {
	let response =
	    test_app.get_route(&format!("hit/{}", &page_id)).await;
	assert!(response.status().is_success());	
    }

    let hits = sqlx::query!(
	r#"
SELECT hits
FROM pages
WHERE page_id = $1
"#,
	page_id)
	.fetch_one(&test_app.db)
	.await
	.expect("Failed to retrieve hits");

    assert_eq!(hits.hits, n_hits);
}
