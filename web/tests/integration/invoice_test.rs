use crate::{authenticated_request, test_request_with_db};
use fake::{Fake, Faker};
use shipwright_db::{DbPool, MIGRATOR, entities::invoices::InvoiceChangeset};

#[sqlx::test(migrator = "MIGRATOR")]
async fn index_page_works_for_authenticated_users(pool: DbPool) {
    authenticated_request::<_, _>(pool.clone(), |request| async move {
        let response = request.get("/invoices").await;

        response.assert_status_ok();
    })
    .await;
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn create_invoice_redirects_and_displays_in_ui(pool: DbPool) {
    let invoice: InvoiceChangeset = Faker.fake();

    test_request_with_db::<_, _>(pool, |request| async move {
        let response = request.post("/invoices").form(&invoice).await;

        response.assert_status_see_other();

        // Manually follow the redirection and assert UI reflects new todo
        let location = response
            .headers()
            .get("location")
            .expect("unable to get redirect location header from response")
            .to_str()
            .unwrap();

        let response = request.get(location).await;

        response.assert_text_contains(invoice.amount.unwrap().to_string());
    })
    .await
}
//
// #[sqlx::test(migrator = "MIGRATOR")]
// async fn create_persists_todo_in_database(pool: DbPool) {
//     let todo: TodoChangeset = Faker.fake();
//
//     test_request_with_db::<_, _>(pool.clone(), |request| async move {
//         let _response = request.post("/todos").form(&todo).await;
//
//         let saved_todo = sqlx::query_as!(
//             Todo,
//             "SELECT * FROM todos WHERE description = ?",
//             todo.description
//         )
//         .fetch_optional(&pool)
//         .await
//         .unwrap();
//
//         assert!(saved_todo.is_some())
//     })
//     .await
// }
//
// #[sqlx::test(migrator = "MIGRATOR")]
// async fn create_throws_422_for_invalid_form_input(pool: DbPool) {
//     test_request_with_db::<_, _>(pool, |request| async move {
//         let response = request
//             .post("/todos")
//             .form(&TodoChangeset {
//                 description: "".to_string(),
//             })
//             .await;
//
//         response.assert_status_unprocessable_entity();
//     })
//     .await
// }
//
// #[sqlx::test(migrator = "MIGRATOR", fixtures("todos"))]
// async fn delete_works(pool: DbPool) {
//     let todo = Todo {
//         id: 1,
//         description: "buy milk".to_string(),
//     };
//
//     test_request_with_db::<_, _>(pool.clone(), |request| async move {
//         let response = request.delete(&format!("/todos/{}", todo.id)).await;
//         response.assert_status_see_other();
//
//         let location = response
//             .headers()
//             .get("location")
//             .expect("unable to get redirect location header")
//             .to_str()
//             .unwrap();
//
//         let response = request.get(location).await;
//
//         let deleted_todo = sqlx::query_as!(Todo, "SELECT * FROM todos WHERE id = ?", todo.id)
//             .fetch_optional(&pool)
//             .await
//             .unwrap();
//
//         assert!(
//             deleted_todo.is_none(),
//             "the todo should no longer exist in the database"
//         );
//
//         assert!(
//             !response
//                 .text()
//                 .contains(&format!("{}/{}", location, todo.id)),
//             "the todo should no longer exist in the UI"
//         );
//     })
//     .await
// }
//
// #[sqlx::test(migrator = "MIGRATOR", fixtures("todos"))]
// async fn update_works(pool: DbPool) {
//     let todo = Todo {
//         id: 1,
//         description: "buy milk".to_string(),
//     };
//
//     let updated_todo: TodoChangeset = Faker.fake();
//
//     test_request_with_db::<_, _>(pool, |request| async move {
//         let response = request
//             .put(&format!("/todos/{}", todo.id))
//             .form(&updated_todo)
//             .await;
//
//         response.assert_status_see_other();
//
//         // Manually follow the redirection and assert UI reflexts new todo
//         let location = response
//             .headers()
//             .get("location")
//             .expect("unable to get redirect location header from response")
//             .to_str()
//             .unwrap();
//
//         let response = request.get(location).await;
//
//         response.assert_text_contains(updated_todo.description);
//     })
//     .await
// }
