use crate::helper::TestApp;
use uuid::Uuid;
use rand::Rng;
use tokio::time::{Duration, sleep};

// During tests, each visit takes 1 second. I.e. a new
// hit is only counted every second.

#[tokio::test]
async fn hit_increment_works() {
    let test_app = TestApp::spawn().await;
    let page_id = test_app.insert_page().await;

    let n_hits = rand::thread_rng().gen_range(0..5);
    for _ in 0..n_hits {
	let response =
	    test_app.get_route(&format!("hit/{}", &page_id)).await;
	assert!(response.status().is_success());
	sleep(Duration::from_secs(2)).await;
    }

    assert_eq!(get_hits(&test_app.db, page_id).await, n_hits);
}

#[tokio::test]
async fn register_and_hit_works() {
    const URL: &str = "https://example.com/";
    let test_app = TestApp::spawn().await;
    let page_id = test_app
	.post_register(&format!("url={URL}"))
	.await
	.json::<Uuid>()
	.await
	.unwrap();
    let n_hits = rand::thread_rng().gen_range(0..5);
    for _ in 0..n_hits {
	let response =
	    test_app.get_route(&format!("hit/{}", &page_id)).await;
	assert!(response.status().is_success());
	sleep(Duration::from_secs(2)).await;
    }

    assert_eq!(get_hits(&test_app.db, page_id).await, n_hits);
}

#[tokio::test]
async fn ignoring_hits_during_a_single_visit_works() {
    let test_app = TestApp::spawn().await;
    let page_id = test_app.insert_page().await;

    // Here the hit/ endpoint is accessed multiple times
    // without any delays in between. The timeouts should
    // be small enough, so that only a single hit gets
    // detected in the single second visit time.
    let n_hits = rand::thread_rng().gen_range(0..4);
    for _ in 0..n_hits {
	let response =
	    test_app.get_route(&format!("hit/{}", &page_id)).await;
	assert!(response.status().is_success());
    }

    assert_eq!(get_hits(&test_app.db, page_id).await, 1);
}

async fn get_hits(db: &sqlx::PgPool, page_id: Uuid) -> i32 {
    let record = sqlx::query!(
	r#"
SELECT hits
FROM pages
WHERE page_id = $1
"#,
	page_id)
	.fetch_one(db)
	.await
	.expect("Failed to retrieve hits");
    record.hits
}
