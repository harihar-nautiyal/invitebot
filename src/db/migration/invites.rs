use once_cell::sync::Lazy;

pub static INVITES: Lazy<String> = Lazy::new(|| {
    r#"
        DEFINE TABLE invites TYPE NORMAL SCHEMAFULL;
        DEFINE FIELD sent_to ON TABLE invites TYPE record<member>;
        DEFINE FIELD room ON TABLE invites TYPE string;
        DEFINE FIELD created_at ON TABLE invites TYPE datetime;
        DEFINE FIELD updated_at ON TABLE invites TYPE datetime;
"#
    .to_string()
});
