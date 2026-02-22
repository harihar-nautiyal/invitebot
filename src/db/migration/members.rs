use once_cell::sync::Lazy;

pub static MEMBERS: Lazy<String> = Lazy::new(|| {
    r#"
        DEFINE TABLE members TYPE NORMAL SCHEMAFULL;
        DEFINE FIELD address ON TABLE members TYPE string;
        DEFINE FIELD display_name ON TABLE members TYPE option<string>;
        DEFINE FIELD created_at ON TABLE members TYPE datetime;
        DEFINE FIELD updated_at ON TABLE members TYPE datetime VALUE time::now();

        DEFINE INDEX members_address_unique ON TABLE members COLUMNS address UNIQUE;
"#
    .to_string()
});
