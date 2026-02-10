use std::io::{StdinLock, StdoutLock, Write, stdin, stdout};

use axum_boilerplate::db::models::*;
use axum_boilerplate::db::*;
use diesel::prelude::*;

use termion::input::TermRead;

use clap::{Parser, Subcommand};
use tracing::info;

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(subcommand)]
    User(UserCommands),
}

#[derive(Debug, Subcommand)]
enum UserCommands {
    New,
    Show,
    Edit { id: i32 },
    Delete { id: i32 },
}

fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::User(user_command) => match user_command {
            UserCommands::New => {
                create_new_user_from_prompt();
            }
            UserCommands::Show => {
                show_users();
            }
            UserCommands::Edit { id } => {
                edit_user(*id);
            }
            UserCommands::Delete { id } => {
                delete_user_by_id(*id);
            }
        },
    };
}

fn show_users() {
    use axum_boilerplate::db::schema::users::dsl::*;

    let connection = &mut establish_connection();
    let results = users
        .limit(5)
        .select(User::as_select())
        .load(connection)
        .expect("Error loading users");

    println!("Displaying {} users", results.len());
    for user in results {
        println!("{:#?}", user);
    }
}

fn create_new_user_from_prompt() {
    use axum_boilerplate::db::schema::users;

    let connection = &mut establish_connection();

    let stdout = stdout();
    let mut stdout = stdout.lock();
    let stdin = stdin();
    let mut stdin = stdin.lock();

    stdout.write_all(b"username: ").unwrap();
    stdout.flush().unwrap();
    let username = stdin
        .read_line()
        .unwrap()
        .expect("Username cannot be blank");

    let hashed_password = prompt_and_hash_password(&mut stdin, &mut stdout);
    let email = prompt_email(&mut stdin, &mut stdout);

    let new_user = NewUser {
        username: &username.trim(),
        hashed_password: hashed_password.as_ref().map(|x| x.as_ref()),
        email: email.as_ref().map(|x| x.as_ref()),
    };

    let user = diesel::insert_into(users::table)
        .values(&new_user)
        .returning(User::as_returning())
        .get_result(connection)
        .expect("error saving user");

    info!("created: {user:#?}");
}

fn prompt_and_hash_password(stdin: &mut StdinLock, stdout: &mut StdoutLock) -> Option<String> {
    stdout.write_all(b"password: ").unwrap();
    stdout.flush().unwrap();
    let password = stdin.read_passwd(stdout).unwrap();

    stdout.write_all(b"\n").unwrap();
    stdout.flush().unwrap();

    match password {
        Some(password) => {
            if password.len() == 0 {
                return None;
            }
            let hashed_password = bcrypt::hash(password.trim(), bcrypt::DEFAULT_COST).unwrap();
            Some(hashed_password)
        }
        None => None,
    }
}

fn prompt_email(stdin: &mut StdinLock, stdout: &mut StdoutLock) -> Option<EmailAddress> {
    stdout.write_all(b"email: ").unwrap();
    stdout.flush().unwrap();
    let raw_email = stdin.read_line().unwrap();

    stdout.write_all(b"\n").unwrap();
    stdout.flush().unwrap();

    match raw_email {
        Some(raw_email) => {
            let email = raw_email.trim();
            if email.len() == 0 {
                return None;
            }
            Some(EmailAddress::new(email).unwrap())
        }
        None => None,
    }
}

fn edit_user(id: i32) {
    use axum_boilerplate::db::schema::users::dsl::users;

    let connection = &mut establish_connection();

    let stdout = stdout();
    let mut stdout = stdout.lock();
    let stdin = stdin();
    let mut stdin = stdin.lock();

    let hashed_password = prompt_and_hash_password(&mut stdin, &mut stdout);
    let email = prompt_email(&mut stdin, &mut stdout);

    let mut user: User = users
        .find(id)
        .first(connection)
        .expect("No user with that id");

    if let Some(_) = hashed_password {
        user.hashed_password = hashed_password;
    }

    if let Some(raw_email) = email {
        user.email = Some(EmailAddress::new(raw_email.as_str()).unwrap());
    }

    let user = diesel::update(users)
        .set(user)
        .returning(User::as_returning())
        .get_result(connection)
        .unwrap();

    println!("{user:#?}");
}

fn delete_user_by_id(id_to_delete: i32) {
    use axum_boilerplate::db::schema::users::dsl::*;

    let connection = &mut establish_connection();

    let num_deleted = diesel::delete(users.filter(id.eq(id_to_delete)))
        .execute(connection)
        .expect("Error while deleting user");

    println!("delected {num_deleted} users");
}
