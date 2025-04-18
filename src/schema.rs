diesel::table! {
    Author (Id) {
        Id -> Integer,
        Name -> Nullable<Text>,
    }
}

diesel::table! {
    Books (id) {
        title -> Text,
        price -> Nullable<Float>,
        bought -> Nullable<Timestamp>,
        finished -> Nullable<Timestamp>,
        added -> Nullable<Timestamp>,
        AuthorFK -> Nullable<Integer>,
        id -> Integer,
    }
}

diesel::joinable!(Books -> Author (AuthorFK));

diesel::allow_tables_to_appear_in_same_query!(
    Author,
    Books,
);
