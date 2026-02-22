use once_cell::sync::Lazy;

pub static USERS: Lazy<String> = Lazy::new(|| {
    r#"
        DEFINE TABLE users TYPE NORMAL SCHEMAFULL;
        DEFINE FIELD address ON TABLE users TYPE string;
        DEFINE FIELD display_name ON TABLE users TYPE option<string>;
        DEFINE FIELD rooms ON TABLE users TYPE array<record<rooms>>;
        DEFINE FIELD created_at ON TABLE users TYPE datetime;
        DEFINE FIELD updated_at ON TABLE users TYPE datetime;
        DEFINE INDEX users_address_unique ON TABLE users COLUMNS address UNIQUE;
"#
    .to_string()
});
