use super::helpers::authenticated_request;
use {{ db_crate_name }}::{DbPool, MIGRATOR, entities::{{ entity_plural_name }}::{% raw %}{{% endraw %}{{ entity_struct_name }}, {{ entity_struct_name }}Changeset}{% raw %}}{% endraw %};
use fake::{Fake, Faker};

#[sqlx::test(migrator = "MIGRATOR")]
async fn {{ entity_plural_name }}_index_page_works_for_authenticated_users(pool: DbPool) {
    authenticated_request::<_, _>(pool.clone(), |request| async move {
        let response = request.get("/{{ entity_plural_name }}").await;
        response.assert_status_ok();
    })
    .await;
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn create_{{ entity_singular_name }}_works(pool: DbPool) {
    let {{ entity_singular_name }}: {{ entity_struct_name }}Changeset = Faker.fake();

    authenticated_request::<_, _>(pool.clone(), |request| async move {
        let response = request.post("/{{ entity_plural_name }}").form(&{{ entity_singular_name }}).await;
        response.assert_status_see_other();

        // Follow the redirection and verify the {{ entity_singular_name }} is shown
        let location = response
            .headers()
            .get("location")
            .expect("unable to get redirect location header")
            .to_str()
            .unwrap();

        let response = request.get(location).await;
        {% for field in changeset_struct_fields -%}
        response.assert_text_contains(&{{ entity_singular_name }}.{{ field.name }});
        {%- endfor %}
    })
    .await;
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn create_{{ entity_singular_name }}_persists_in_database(pool: DbPool) {
    let {{ entity_singular_name }}: {{ entity_struct_name }}Changeset = Faker.fake();

    authenticated_request::<_, _>(pool.clone(), |request| async move {
        let _response = request.post("/{{ entity_plural_name }}").form(&{{ entity_singular_name }}).await;

        let saved_{{ entity_singular_name }} = sqlx::query_as!(
            {{ entity_struct_name }},
            "SELECT * FROM {{ entity_plural_name }} WHERE {% for field in changeset_struct_fields %}{% if forloop.first %}{{ field.name }} = ?{% else %} AND {{ field.name }} = ?{% endif %}{% endfor %}",
            {% for field in changeset_struct_fields -%}
            {{ entity_singular_name }}.{{ field.name }}{% unless forloop.last %},{% endunless %}
            {%- endfor %}
        )
        .fetch_optional(&pool)
        .await
        .unwrap();

        assert!(saved_{{ entity_singular_name }}.is_some(), "{{ entity_singular_name }} should be saved in database");
    })
    .await;
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn invalid_create_{{ entity_singular_name }}_returns_422(pool: DbPool) {
    authenticated_request::<_, _>(pool.clone(), |request| async move {
        let response = request
            .post("/{{ entity_plural_name }}")
            .form(&{{ entity_struct_name }}Changeset {
                {% for field in changeset_struct_fields -%}
                {{ field.name }}: "".to_string(),
                {%- endfor %}
            })
            .await;

        response.assert_status_unprocessable_entity();
    })
    .await;
}

#[sqlx::test(migrator = "MIGRATOR", fixtures("{{ entity_plural_name }}"))]
async fn show_{{ entity_singular_name }}_works(pool: DbPool) {
    authenticated_request::<_, _>(pool.clone(), |request| async move {
        let response = request.get("/{{ entity_plural_name }}/1").await;
        response.assert_status_ok();
        {% for field in changeset_struct_fields -%}
        response.assert_text_contains("{{ field.name }}"); // This should match your fixture data
        {%- endfor %}
    })
    .await;
}

#[sqlx::test(migrator = "MIGRATOR", fixtures("{{ entity_plural_name }}"))]
async fn update_{{ entity_singular_name }}_works(pool: DbPool) {
    let updated_{{ entity_singular_name }} = {{ entity_struct_name }}Changeset {
        {% for field in changeset_struct_fields -%}
        {{ field.name }}: "updated {{ field.name }}".to_string(),
        {%- endfor %}
    };

    authenticated_request::<_, _>(pool.clone(), |request| async move {
        let response = request.put("/{{ entity_plural_name }}/1").form(&updated_{{ entity_singular_name }}).await;
        response.assert_status_see_other();

        // Follow the redirection and verify the update
        let location = response
            .headers()
            .get("location")
            .expect("unable to get redirect location header")
            .to_str()
            .unwrap();

        let response = request.get(location).await;
        {% for field in changeset_struct_fields -%}
        response.assert_text_contains(&updated_{{ entity_singular_name }}.{{ field.name }});
        {%- endfor %}
    })
    .await;
}

#[sqlx::test(migrator = "MIGRATOR", fixtures("{{ entity_plural_name }}"))]
async fn delete_{{ entity_singular_name }}_works(pool: DbPool) {
    authenticated_request::<_, _>(pool.clone(), |request| async move {
        let response = request.delete("/{{ entity_plural_name }}/1").await;
        response.assert_status_see_other();

        // Follow the redirection and verify the {{ entity_singular_name }} is gone
        let location = response
            .headers()
            .get("location")
            .expect("unable to get redirect location header")
            .to_str()
            .unwrap();

        let response = request.get(location).await;
        {% for field in changeset_struct_fields -%}
        assert_ne!(response.text(), "{{ field.name }}"); // This should match your fixture data
        {%- endfor %}

        // Verify the {{ entity_singular_name }} is deleted from the database
        let deleted_{{ entity_singular_name }} = sqlx::query_as!(
            {{ entity_struct_name }},
            "SELECT * FROM {{ entity_plural_name }} WHERE id = ?",
            1
        )
        .fetch_optional(&pool)
        .await
        .unwrap();

        assert!(
            deleted_{{ entity_singular_name }}.is_none(),
            "{{ entity_singular_name }} should be deleted from database"
        );
    })
    .await;
}
