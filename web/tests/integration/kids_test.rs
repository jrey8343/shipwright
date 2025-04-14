use super::helpers::authenticated_request;
use fake::{Fake, Faker};
use shipwright_db::{
    DbPool, MIGRATOR,
    entities::kids::{Kid, KidChangeset},
};

#[sqlx::test(migrator = "MIGRATOR")]
async fn kids_index_page_works_for_authenticated_users(pool: DbPool) {
    authenticated_request::<_, _>(pool.clone(), |request| async move {
        let response = request.get("/kids").await;
        response.assert_status_ok();
    })
    .await;
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn create_kid_works(pool: DbPool) {
    let kid: KidChangeset = Faker.fake();

    authenticated_request::<_, _>(pool.clone(), |request| async move {
        let response = request.post("/kids").form(&kid).await;
        response.assert_status_see_other();

        // Follow the redirection and verify the kid is shown
        let location = response
            .headers()
            .get("location")
            .expect("unable to get redirect location header")
            .to_str()
            .unwrap();

        let response = request.get(location).await;
        response.assert_text_contains(&kid.name);
        response.assert_text_contains(&kid.nickname);
        response.assert_text_contains(&kid.date_of_birth);
    })
    .await;
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn create_kid_persists_in_database(pool: DbPool) {
    let kid: KidChangeset = Faker.fake();

    authenticated_request::<_, _>(pool.clone(), |request| async move {
        let _response = request.post("/kids").form(&kid).await;

        let saved_kid = sqlx::query_as!(
            Kid,
            "SELECT * FROM kids WHERE name = ? AND nickname = ? AND date_of_birth = ?",
            kid.name,
            kid.nickname,
            kid.date_of_birth
        )
        .fetch_optional(&pool)
        .await
        .unwrap();

        assert!(saved_kid.is_some(), "kid should be saved in database");
    })
    .await;
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn invalid_create_kid_returns_422(pool: DbPool) {
    authenticated_request::<_, _>(pool.clone(), |request| async move {
        let response = request
            .post("/kids")
            .form(&KidChangeset {
                name: "".to_string(),
                nickname: "".to_string(),
                date_of_birth: "".to_string(),
                updated_at: OffsetDateTime::now_utc(),
            })
            .await;

        response.assert_status_unprocessable_entity();
    })
    .await;
}

#[sqlx::test(migrator = "MIGRATOR", fixtures("kids"))]
async fn show_kid_works(pool: DbPool) {
    authenticated_request::<_, _>(pool.clone(), |request| async move {
        let response = request.get("/kids/1").await;
        response.assert_status_ok();
        response.assert_text_contains("name"); // This should match your fixture dataresponse.assert_text_contains("nickname"); // This should match your fixture dataresponse.assert_text_contains("date_of_birth"); // This should match your fixture data
    })
    .await;
}

#[sqlx::test(migrator = "MIGRATOR", fixtures("kids"))]
async fn update_kid_works(pool: DbPool) {
    let updated_kid = KidChangeset {
        name: "updated name".to_string(),
        nickname: "updated nickname".to_string(),
        date_of_birth: "updated date_of_birth".to_string(),
    };

    authenticated_request::<_, _>(pool.clone(), |request| async move {
        let response = request.put("/kids/1").form(&updated_kid).await;
        response.assert_status_see_other();

        // Follow the redirection and verify the update
        let location = response
            .headers()
            .get("location")
            .expect("unable to get redirect location header")
            .to_str()
            .unwrap();

        let response = request.get(location).await;
        response.assert_text_contains(&updated_kid.name);
        response.assert_text_contains(&updated_kid.nickname);
        response.assert_text_contains(&updated_kid.date_of_birth);
    })
    .await;
}

#[sqlx::test(migrator = "MIGRATOR", fixtures("kids"))]
async fn delete_kid_works(pool: DbPool) {
    authenticated_request::<_, _>(pool.clone(), |request| async move {
        let response = request.delete("/kids/1").await;
        response.assert_status_see_other();

        // Follow the redirection and verify the kid is gone
        let location = response
            .headers()
            .get("location")
            .expect("unable to get redirect location header")
            .to_str()
            .unwrap();

        let response = request.get(location).await;
        assert_ne!(response.text(), "name"); // This should match your fixture dataassert_ne!(response.text(), "nickname"); // This should match your fixture dataassert_ne!(response.text(), "date_of_birth"); // This should match your fixture data

        // Verify the kid is deleted from the database
        let deleted_kid = sqlx::query_as!(Kid, "SELECT * FROM kids WHERE id = ?", 1)
            .fetch_optional(&pool)
            .await
            .unwrap();

        assert!(deleted_kid.is_none(), "kid should be deleted from database");
    })
    .await;
}
