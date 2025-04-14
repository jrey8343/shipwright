use super::helpers::authenticated_request;
use fake::{Fake, Faker};
use shipwright_db::entities::todo::TodoChangeset;
use shipwright_db::{DbPool, MIGRATOR};

#[sqlx::test(migrator = "MIGRATOR")]
async fn todos_index_page_works_for_authenticated_users(pool: DbPool) {
    authenticated_request::<_, _>(pool.clone(), |request| async move {
        let response = request.get("/todos").await;
        response.assert_status_ok();
    })
    .await;
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn create_todo_works(pool: DbPool) {
    let todo: TodoChangeset = Faker.fake();

    authenticated_request::<_, _>(pool.clone(), |request| async move {
        let response = request.post("/todos").form(&todo).await;
        response.assert_status_see_other();

        // Follow the redirection and verify the todo is shown
        let location = response
            .headers()
            .get("location")
            .expect("unable to get redirect location header")
            .to_str()
            .unwrap();

        let response = request.get(location).await;
        response.assert_text_contains(&todo.description);
    })
    .await;
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn create_todo_persists_todo_in_database(pool: DbPool) {
    let todo: TodoChangeset = Faker.fake();

    authenticated_request::<_, _>(pool.clone(), |request| async move {
        let _response = request.post("/todos").form(&todo).await;

        let saved_todo = sqlx::query_as!(
            shipwright_db::entities::todo::Todo,
            "SELECT * FROM todos WHERE description = ?",
            todo.description
        )
        .fetch_optional(&pool)
        .await
        .unwrap();

        assert!(saved_todo.is_some(), "todo should be saved in database");
    })
    .await;
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn invalid_create_todo_returns_422(pool: DbPool) {
    authenticated_request::<_, _>(pool.clone(), |request| async move {
        let response = request
            .post("/todos")
            .form(&TodoChangeset {
                description: "".to_string(),
            })
            .await;

        response.assert_status_unprocessable_entity();
    })
    .await;
}

#[sqlx::test(migrator = "MIGRATOR", fixtures("todos"))]
async fn show_todo_works(pool: DbPool) {
    authenticated_request::<_, _>(pool.clone(), |request| async move {
        let response = request.get("/todos/1").await;
        response.assert_status_ok();
        response.assert_text_contains("buy milk");
    })
    .await;
}

#[sqlx::test(migrator = "MIGRATOR", fixtures("todos"))]
async fn update_works(pool: DbPool) {
    let updated_todo = TodoChangeset {
        description: "buy organic milk".to_string(),
    };

    authenticated_request::<_, _>(pool.clone(), |request| async move {
        let response = request.put("/todos/1").form(&updated_todo).await;

        response.assert_status_see_other();

        // Follow the redirection and verify the update
        let location = response
            .headers()
            .get("location")
            .expect("unable to get redirect location header")
            .to_str()
            .unwrap();

        let response = request.get(location).await;
        response.assert_text_contains(&updated_todo.description);
    })
    .await;
}

#[sqlx::test(migrator = "MIGRATOR", fixtures("todos"))]
async fn delete_todo_works(pool: DbPool) {
    authenticated_request::<_, _>(pool.clone(), |request| async move {
        let response = request.delete("/todos/1").await;
        response.assert_status_see_other();

        // Follow the redirection and verify the todo is gone
        let location = response
            .headers()
            .get("location")
            .expect("unable to get redirect location header")
            .to_str()
            .unwrap();

        let response = request.get(location).await;
        assert_ne!(response.text(), "buy milk");

        // Verify the todo is deleted from the database
        let deleted_todo = sqlx::query_as!(
            shipwright_db::entities::todo::Todo,
            "SELECT * FROM todos WHERE id = ?",
            1
        )
        .fetch_optional(&pool)
        .await
        .unwrap();

        assert!(
            deleted_todo.is_none(),
            "todo should be deleted from database"
        );
    })
    .await;
}
