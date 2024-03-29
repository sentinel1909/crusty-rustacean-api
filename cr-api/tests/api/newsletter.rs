// tests/api/newsletter.rs

use crate::helpers::{assert_is_redirect_to, spawn_app, ConfirmationLinks, TestApp};
use fake::faker::internet::en::SafeEmail;
use fake::faker::name::en::Name;
use fake::Fake;
use std::time::Duration;
use wiremock::matchers::{any, method, path};
use wiremock::{Mock, ResponseTemplate};

async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    let name: String = Name().fake();
    let email: String = SafeEmail().fake();
    let body = serde_urlencoded::to_string(serde_json::json!({
    "name": name,
    "email": email
    }))
    .unwrap();

    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;
    app.post_subscriptions(body)
        .await
        .error_for_status()
        .unwrap();

    // We now inspect the requests received by the mock Postmark server
    // to retrieve the confirmation link and return it
    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();
    app.get_confirmation_links(email_request)
}

async fn create_confirmed_subscriber(app: &TestApp) {
    // We can then reuse the same helper and just add
    // an extra step to actually call the confirmation link!
    let confirmation_link = create_unconfirmed_subscriber(app).await;
    reqwest::get(confirmation_link.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    // Arrange
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    // Act - Part 1 - Submit newsletter form
    let newsletter_request_body = serde_json::json!({
      "title": "Newsletter title",
      "text_content": "Newsletter body as plain text",
      "html_content": "<p>Newsletter body as HTML</p>",
      "idempotency_key": uuid::Uuid::new_v4().to_string()
    });
    let response = app.post_publish_newsletter(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletter");

    // Act - Part 2 - Follow the redirect
    let html_page = app.get_publish_newsletter_html().await;
    assert!(
        html_page.contains("The newsletter issue has been accepted - emails will go out shortly.")
    );
    app.dispatch_all_pending_emails().await;
    // Mock verifies on Drop that we haven't sent the newsletter email
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    // Arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act - Part 1 - Submit newsletter form
    let newsletter_request_body = serde_json::json!({
      "title": "Newsletter title",
      "text_content": "Newsletter body as plain text",
      "html_content": "<p>Newsletter body as HTML</p>",
      "idempotency_key": uuid::Uuid::new_v4().to_string()
    });
    let response = app.post_publish_newsletter(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletter");

    // Act - Part 2 - Follow the redirect
    let html_page = app.get_publish_newsletter_html().await;
    assert!(
        html_page.contains("The newsletter issue has been accepted - emails will go out shortly.")
    );
    app.dispatch_all_pending_emails().await;
    // Mock verifies on Drop that we haven't sent the newsletter email
}

#[tokio::test]
async fn you_must_be_logged_in_to_see_the_newsletter_form() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app.get_publish_newsletter().await;

    // Assert
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn you_must_be_logged_in_to_publish_a_newsletter() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });
    let response = app.post_publish_newsletter(&newsletter_request_body).await;

    // Assert
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn invalid_newsletter_form_data_is_rejected() {
    // Arrange
    let app = spawn_app().await;
    let test_cases = vec![
        (
            serde_json::json!({
                "title": "tes",
                "text_content": "Newsletter body as plain text",
                "html_content": "<p>Newsletter body as HTML</p>",
                "idempotency_key": uuid::Uuid::new_v4().to_string()
            }),
            "title has less than 5 characters",
        ),
        (
            serde_json::json!({
                "title": "Newsletter title",
                "text_content": "tes",
                "html_content": "<p>Newsletter body as HTML</p>",
                "idempotency_key": uuid::Uuid::new_v4().to_string()
            }),
            "text_content has less than 5 characters",
        ),
        (
            serde_json::json!({
                "title": "Newsletter title",
                "text_content": "Newsletter body as plain text",
                "html_content": "tes",
                "idempotency_key": uuid::Uuid::new_v4().to_string()
            }),
            "html_content has less than 5 characters",
        ),
    ];

    app.test_user.login(&app).await;

    // Act
    for (invalid_body, error_message) in test_cases {
        let _ = app
            .post_publish_newsletter(&invalid_body)
            .await
            .text()
            .await;

        let html = app.get_publish_newsletter_html().await;

        // Assert
        assert!(
            html.contains("Part of the form body has less than 5 characters"),
            "The API did not fail when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn newsletter_creation_is_idempotent() {
    // Arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act - Part 1 - Submit newsletter form
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });
    let response = app.post_publish_newsletter(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletter");

    // Act - Part 2 - follow the redirect
    let html_page = app.get_publish_newsletter_html().await;
    assert!(
        html_page.contains("The newsletter issue has been accepted - emails will go out shortly.")
    );

    // Act - Part 3 - Submit newsletter form **again**
    let response = app.post_publish_newsletter(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletter");

    // Act - Part 4 - Follow the redirect
    let html_page = app.get_publish_newsletter_html().await;
    assert!(
        html_page.contains("The newsletter issue has been accepted - emails will go out shortly.")
    );
    app.dispatch_all_pending_emails().await;
    // Mock verifies on Drop that we have the the newsletter email **once**
}

#[tokio::test]
async fn concurrent_form_submission_is_handled_gracefully() {
    // Arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;
    Mock::given(path("/email"))
        .and(method("POST"))
        // Setting a long delay to ensure that the second request
        // arrives before the first one completes
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(2)))
        .expect(1)
        .mount(&app.email_server)
        .await;
    // Act - Submit two newsletter forms concurrently
    let newsletter_request_body = serde_json::json!({
    "title": "Newsletter title",
    "text_content": "Newsletter body as plain text",
    "html_content": "<p>Newsletter body as HTML</p>",
    "idempotency_key": uuid::Uuid::new_v4().to_string()
    });
    let response1 = app.post_publish_newsletter(&newsletter_request_body);
    let response2 = app.post_publish_newsletter(&newsletter_request_body);
    let (response1, response2) = tokio::join!(response1, response2);
    assert_eq!(response1.status(), response2.status());
    assert_eq!(
        response1.text().await.unwrap(),
        response2.text().await.unwrap()
    );
    app.dispatch_all_pending_emails().await
    // Mock verifies on Drop that we have sent the newsletter email **once**
}

#[tokio::test]
async fn old_idempotency_entries_are_cleaned_up() {
    // Arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;

    // Get the new subscriber's user_id
    let user = sqlx::query!("SELECT user_id FROM users")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");
    let user_id = user.user_id;

    // Manually create a test idempotency record. No need for actual data
    // because we're just testing the cleaner cleans old ones out.
    sqlx::query!(
        r#"
        INSERT INTO idempotency (
            idempotency_key,
            user_id,
            created_at 
        )
        VALUES ($1, $2, now() - interval '6 days')
        "#,
        uuid::Uuid::new_v4().to_string(),
        &user_id,
    )
    .execute(&app.db_pool)
    .await
    .expect("Couldn't create an idempotency entry");

    // Create a second one that won't be cleaned because it's too new
    sqlx::query!(
        r#"
        INSERT INTO idempotency (
            idempotency_key,
            user_id,
            created_at 
        )
        VALUES ($1, $2, now())
        "#,
        uuid::Uuid::new_v4().to_string(),
        user_id,
    )
    .execute(&app.db_pool)
    .await
    .expect("Couldn't create an idempotency entry");

    // Act
    app.clean_up_idempotency().await;

    // struct Count {
    //     value: Option<i64>,
    // }

    // Assert
    let count = sqlx::query!(
        r#"
        SELECT COUNT (*) as "value"
        FROM idempotency
        "#
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("Couldn't get the idempotency count");

    assert_eq!(1, count.value.unwrap());
}
