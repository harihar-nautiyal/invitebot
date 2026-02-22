use once_cell::sync::Lazy;

pub static ROOMS: Lazy<String> = Lazy::new(|| {
    r#"
        DEFINE TABLE rooms TYPE NORMAL SCHEMAFULL;
        DEFINE FIELD title ON TABLE rooms TYPE string;
        DEFINE FIELD description ON TABLE rooms TYPE string;
        DEFINE FIELD address ON TABLE rooms TYPE string;
        DEFINE FIELD founder ON TABLE rooms TYPE string;
        DEFINE FIELD members ON TABLE rooms TYPE array<record<member>>;
        DEFINE FIELD invited_members ON TABLE rooms TYPE array<record<member>>;
        DEFINE FIELD created_at ON TABLE rooms TYPE datetime;
        DEFINE FIELD updated_at ON TABLE rooms TYPE datetime VALUE time::now();
"#
    .to_string()
});
