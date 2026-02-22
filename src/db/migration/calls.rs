use once_cell::sync::Lazy;

pub static CALLS: Lazy<String> = Lazy::new(|| {
    r#"
        DEFINE TABLE calls TYPE NORMAL SCHEMAFULL;
        DEFINE FIELD user ON TABLE calls TYPE record<users>;
        DEFINE FIELD room ON TABLE calls TYPE record<rooms>;
        DEFINE FIELD event_id ON TABLE calls TYPE string;
        DEFINE FIELD command ON TABLE calls TYPE string;
        DEFINE FIELD status ON TABLE calls TYPE "pending" | "completed" | "rejected";
        DEFINE FIELD created_at ON TABLE calls TYPE datetime;
        DEFINE FIELD updated_at ON TABLE calls TYPE datetime VALUE time::now();
"#
    .to_string()
});
