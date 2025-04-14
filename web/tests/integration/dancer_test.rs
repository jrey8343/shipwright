use super::helpers::authenticated_request;
use fake::{Fake, Faker};
use shipwright_db::{
    DbPool, MIGRATOR,
    entities::dancers::{Dancer, DancerChangeset},
};

#[sqlx::test(migrator = "MIGRATOR")]
async fn dancers_index_page_works_for_authenticated_users(pool: DbPool) {
    authenticated_request::<_, _>(pool.clone(), |request| async move {
        let response = request.get("/dancers").await;
        response.assert_status_ok();
    })
    .await;
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn create_dancer_works(pool: DbPool) {
    let dancer: DancerChangeset = Faker.fake();

    authenticated_request::<_, _>(pool.clone(), |request| async move {
        let response = request.post("/dancers").form(&dancer).await;
        response.assert_status_see_other();

        // Follow the redirection and verify the dancer is shown
        let location = response
            .headers()
            .get("location")
            .expect("unable to get redirect location header")
            .to_str()
            .unwrap();

        let response = request.get(location).await;

        response.assert_text_contains(&dancer.name);

        response.assert_text_contains(&dancer.email);

        response.assert_text_contains(&dancer.dance_style);
    })
    .await;
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn create_dancer_persists_in_database(pool: DbPool) {
    let dancer: DancerChangeset = Faker.fake();

    authenticated_request::<_, _>(pool.clone(), |request| async move {
        let _response = request.post("/dancers").form(&dancer).await;

        let saved_dancer = sqlx::query_as!(
            Dancer,
            "SELECT * FROM dancers WHERE name = ? AND email = ? AND dance_style = ?",
            dancer.name,
            dancer.email,
            dancer.dance_style
        )
        .fetch_optional(&pool)
        .await
        .unwrap();

        assert!(saved_dancer.is_some(), "dancer should be saved in database");
    })
    .await;
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn invalid_create_dancer_returns_422(pool: DbPool) {
    authenticated_request::<_, _>(pool.clone(), |request| async move {
        let response = request
            .post("/dancers")
            .form(&DancerChangeset {
                name: "".to_string(),

                email: "".to_string(),

                dance_style: "".to_string(),
            })
            .await;

        response.assert_status_unprocessable_entity();
    })
    .await;
}

#[sqlx::test(migrator = "MIGRATOR", fixtures("dancers"))]
async fn show_dancer_works(pool: DbPool) {
    authenticated_request::<_, _>(pool.clone(), |request| async move {
        let response = request.get("/dancers/1").await;
        response.assert_status_ok();

        response.assert_text_contains("name"); // This should match your fixture data

        response.assert_text_contains("email"); // This should match your fixture data

        response.assert_text_contains("dance_style"); // This should match your fixture data
    })
    .await;
}

#[sqlx::test(migrator = "MIGRATOR", fixtures("dancers"))]
async fn update_dancer_works(pool: DbPool) {
    let updated_dancer = DancerChangeset {
        name: "updated name".to_string(),
        email: "updated@example.com".to_string(),
        dance_style: "updated dance_style".to_string(),
    };

    authenticated_request::<_, _>(pool.clone(), |request| async move {
        let response = request.put("/dancers/1").form(&updated_dancer).await;
        response.assert_status_see_other();

        // Follow the redirection and verify the update
        let location = response
            .headers()
            .get("location")
            .expect("unable to get redirect location header")
            .to_str()
            .unwrap();

        let response = request.get(location).await;

        response.assert_text_contains(&updated_dancer.name);

        response.assert_text_contains(&updated_dancer.email);

        response.assert_text_contains(&updated_dancer.dance_style);
    })
    .await;
}

#[sqlx::test(migrator = "MIGRATOR", fixtures("dancers"))]
async fn delete_dancer_works(pool: DbPool) {
    authenticated_request::<_, _>(pool.clone(), |request| async move {
        let response = request.delete("/dancers/1").await;
        response.assert_status_see_other();

        // Follow the redirection and verify the dancer is gone
        let location = response
            .headers()
            .get("location")
            .expect("unable to get redirect location header")
            .to_str()
            .unwrap();

        let response = request.get(location).await;

        assert_ne!(response.text(), "name"); // This should match your fixture data

        assert_ne!(response.text(), "email"); // This should match your fixture data

        assert_ne!(response.text(), "dance_style"); // This should match your fixture data

        // Verify the dancer is deleted from the database
        let deleted_dancer = sqlx::query_as!(Dancer, "SELECT * FROM dancers WHERE id = ?", 1)
            .fetch_optional(&pool)
            .await
            .unwrap();

        assert!(
            deleted_dancer.is_none(),
            "dancer should be deleted from database"
        );
    })
    .await;
}
