use log::error;
use sqlx::{Pool, Postgres};

pub async fn initial_setup(pool: &Pool<Postgres>) {
    let initial_org_title = dotenv::var("INITIAL_ORG").expect("INITIAL_ORG is not configured");
    let initial_user_name =
        dotenv::var("INITIAL_USER").expect("INITIAL_USERNAME is not configured");
    let initial_user_password =
        dotenv::var("INITIAL_PASSWORD").expect("INITIAL_PASSWORD is not configured");

    let Ok(mut tx) = pool.begin().await else {
        panic!("could not start init transaction")
    };

    let Ok(created_org) = sqlx::query!(
        r#"INSERT INTO orgs (title) VALUES ($1) RETURNING id"#,
        initial_org_title,
    )
    .fetch_one(&mut *tx)
    .await
    else {
        panic!("could not create initial org")
    };

    let Ok(created_user) = sqlx::query!(
        r#"INSERT INTO users (username, password, org_id) VALUES ($1, $2, $3) RETURNING id"#,
        initial_user_name,
        initial_user_password,
        created_org.id,
    )
    .fetch_one(&mut *tx)
    .await
    else {
        panic!("could not create initial user")
    };

    // role assginment
    let _ = sqlx::query!(
        r#"INSERT INTO role_assignments (user_id, role) VALUES ($1, 'super_admin')"#,
        created_user.id,
    )
    .execute(&mut *tx)
    .await
    .map_err(|err| {
        error!("{}", err);
        panic!("could not create initial role assignment")
    });

    let _ = sqlx::query!("UPDATE config SET is_initialized=TRUE",)
        .execute(&mut *tx)
        .await
        .map_err(|err| {
            error!("{}", err);
            panic!("could set initial config")
        });

    let _ = tx.commit().await.map_err(|err| {
        error!("{}", err);
        panic!("failed to initialize application")
    });
}
