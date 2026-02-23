use once_cell::sync::Lazy;

pub static ROOMS: Lazy<String> = Lazy::new(|| {
    r#"
        DEFINE TABLE rooms TYPE NORMAL SCHEMAFULL;
        DEFINE FIELD title ON TABLE rooms TYPE string;
        DEFINE FIELD description ON TABLE rooms TYPE string;
        DEFINE FIELD address ON TABLE rooms TYPE string;
        DEFINE FIELD founder ON TABLE rooms TYPE record<members>;
        DEFINE FIELD members ON TABLE rooms TYPE array<record<members>>;
        DEFINE FIELD invited_members ON TABLE rooms TYPE array<record<members>>;
        DEFINE FIELD created_at ON TABLE rooms TYPE datetime;
        DEFINE FIELD updated_at ON TABLE rooms TYPE datetime VALUE time::now();

        DEFINE INDEX unique_room_address ON TABLE rooms FIELDS address UNIQUE;

"#
    .to_string()
});
