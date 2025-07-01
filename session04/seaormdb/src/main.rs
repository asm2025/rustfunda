use anyhow::{Result, anyhow};
use dotenvy::dotenv;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, Database, DbConn, DbErr, EntityTrait,
    QueryFilter, Schema, Set,
};
use std::env;

mod entities;
use entities::user::{self, ActiveModel as UserActiveModel, Entity as User, Model as UserModel};

/// Sets up the database connection and runs migrations.
async fn setup_database() -> Result<DbConn> {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = Database::connect(&db_url).await?;
    let schema = Schema::new(db.get_database_backend());
    let create_table_statement = schema.create_table_from_entity(User);

    match db
        .execute(db.get_database_backend().build(&create_table_statement))
        .await
    {
        Ok(_) => println!("Table 'users' created successfully or already exists."),
        Err(e) => return Err(anyhow!(e)),
    }

    Ok(db)
}

async fn seed_users(db: &DbConn) -> Result<()> {
    let users_to_seed = vec![
        UserActiveModel {
            name: Set("Alice".to_owned()),
            email: Set("alice@example.com".to_owned()),
            ..Default::default()
        },
        UserActiveModel {
            name: Set("Bob".to_owned()),
            email: Set("bob@example.com".to_owned()),
            ..Default::default()
        },
    ];

    User::insert_many(users_to_seed)
        .on_conflict(
            // The path to Column is now cleaner thanks to `use entities::user;`
            sea_orm::sea_query::OnConflict::column(user::Column::Email)
                .do_nothing()
                .to_owned(),
        )
        .exec(db)
        .await?;

    println!("Seeded initial users.");
    Ok(())
}

/// Lists all users in the database.
async fn list_all_users(db: &DbConn, context: &str) -> Result<()> {
    println!("\n--- {} ---", context);

    let users: Vec<UserModel> = User::find().all(db).await?;

    if users.is_empty() {
        println!("No users found.");
    } else {
        for user in users {
            println!("{:?}", user);
        }
    }

    Ok(())
}

/// Updates a user's name by their ID.
async fn update_user_name(db: &DbConn, id: i32, new_name: &str) -> Result<UserModel> {
    println!(
        "\n--- Updating user with ID {} to name '{}' ---",
        id, new_name
    );

    let mut user_to_update: UserActiveModel = User::find_by_id(id)
        .one(db)
        .await?
        .ok_or(DbErr::RecordNotFound(format!(
            "User with ID {} not found",
            id
        )))?
        .into();

    user_to_update.name = Set(new_name.to_owned());
    let model = user_to_update.update(db).await?;
    Ok(model)
}

/// Finds users whose name contains a given string.
async fn find_users_by_name(db: &DbConn, search_str: &str) -> Result<()> {
    println!(
        "\n--- Finding users with names containing '{}' ---",
        search_str
    );

    let found_users: Vec<UserModel> = User::find()
        .filter(user::Column::Name.contains(search_str))
        .all(db)
        .await?;

    if found_users.is_empty() {
        println!("No users found matching the criteria.");
    } else {
        for user in found_users {
            println!("{:?}", user);
        }
    }
    Ok(())
}

/// Deletes a user by their ID.
async fn delete_user_by_id(db: &DbConn, id: i32) -> Result<()> {
    println!("\n--- Deleting user with ID {} ---", id);

    let res = User::delete_by_id(id).exec(db).await?;

    if res.rows_affected == 1 {
        println!("Successfully deleted user with ID {}.", id);
    } else {
        println!("Could not find user with ID {} to delete.", id);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    // 1. Setup database and create schema
    let db = setup_database().await?;

    // 2. Seed the database with initial data
    seed_users(&db).await?;

    // 3. List all records
    list_all_users(&db, "Initial list of users").await?;

    // 4. Update a record
    let updated_user = update_user_name(&db, 1, "Alice Smith").await?;
    println!("Updated user: {:?}", updated_user);

    // 5. List records again to see the update
    list_all_users(&db, "Users after update").await?;

    // 6. Find users by name
    find_users_by_name(&db, "Bob").await?;

    // 7. Delete a record
    delete_user_by_id(&db, 2).await?;

    // 8. Final list of users
    list_all_users(&db, "Final list of users after deletion").await?;

    Ok(())
}
