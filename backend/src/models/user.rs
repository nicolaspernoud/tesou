use serde::{Deserialize, Serialize};

use crate::{
    crud_create, crud_delete, crud_delete_all, crud_read, crud_read_all, crud_update, crud_use,
    errors::ServerError, schema::users,
};

macro_rules! trim {
    () => {
        fn trim(&mut self) -> &Self {
            self.name = self.name.trim().to_string();
            self.surname = self.surname.trim().to_string();
            self
        }
    };
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset, Identifiable,
)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub surname: String,
}
impl User {
    trim!();
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub name: String,
    pub surname: String,
}
impl NewUser {
    trim!();
}

crud_use!();
crud_create!(NewUser, User, users,);
crud_read_all!(User, users);
crud_read!(User, users);
crud_update!(User, users,);
crud_delete!(User, users);
crud_delete_all!(User, users);
