use crate::schema::dictionary;
#[derive(Queryable, Debug)]
pub struct Entry {
    pub id: i32,
    pub chord: String,
    pub translation: String,
}
#[derive(Insertable)]
#[table_name = "dictionary"]
pub struct NewEntry {
    pub chord: String,
    pub translation: String,
}
