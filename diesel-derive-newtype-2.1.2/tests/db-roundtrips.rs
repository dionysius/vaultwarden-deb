use diesel::dsl::sql;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_derive_newtype::DieselNewType;

#[derive(Debug, Clone, PartialEq, Eq, Hash, DieselNewType)]
pub struct MyIdString(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, DieselNewType)]
pub struct MyI32(i32);

#[derive(Debug, Clone, PartialEq, Eq, Hash, DieselNewType)]
pub struct MyNullableString(Option<String>);

#[derive(Debug, Clone, PartialEq, Eq, Hash, DieselNewType)]
pub struct MyNullableI32(Option<i32>);

#[derive(Debug, Clone, PartialEq, Identifiable, Insertable, Queryable)]
#[diesel(table_name = my_entities)]
pub struct MyEntity {
    id: MyIdString,
    my_i32: MyI32,
    my_nullable_string: MyNullableString,
    my_nullable_i32: MyNullableI32,
    val: i32,
}

#[derive(Debug, Clone, PartialEq, Insertable)]
#[diesel(table_name = my_entities)]
pub struct MyEntityInternalRefs<'a> {
    id: &'a MyIdString,
    my_i32: MyI32,
    my_nullable_string: &'a MyNullableString,
    my_nullable_i32: &'a MyNullableI32,
    val: i32,
}

table! {
    my_entities {
        id -> Text,
        my_i32 -> Integer,
        my_nullable_string -> Nullable<Text>,
        my_nullable_i32 -> Nullable<Integer>,
        val -> Integer,
    }
}

#[cfg(test)]
fn setup() -> SqliteConnection {
    let mut conn = SqliteConnection::establish(":memory:").unwrap();
    let setup = sql::<diesel::sql_types::Bool>(
        "CREATE TABLE my_entities (
                id TEXT PRIMARY KEY,
                my_i32 int NOT NULL,
                my_nullable_string TEXT,
                my_nullable_i32 int,
                val Int NOT NULL
         )",
    );
    setup.execute(&mut conn).expect("Can't create table");
    conn
}

#[test]
fn does_roundtrip() {
    let mut conn = setup();
    let obj = MyEntity {
        id: MyIdString("WooHoo".into()),
        my_i32: MyI32(10),
        my_nullable_string: MyNullableString(Some("WooHoo".into())),
        my_nullable_i32: MyNullableI32(Some(10)),
        val: 1,
    };

    diesel::insert_into(my_entities::table)
        .values(&obj)
        .execute(&mut conn)
        .expect("Couldn't insert struct into my_entities");

    let found: Vec<MyEntity> = my_entities::table.load(&mut conn).unwrap();
    println!("found: {:?}", found);
    assert_eq!(found[0], obj);
}

#[test]
fn does_roundtrip_with_ref() {
    let mut conn = setup();
    let obj = MyEntityInternalRefs {
        id: &MyIdString("WooHoo".into()),
        my_i32: MyI32(10),
        my_nullable_string: &MyNullableString(Some("WooHoo".into())),
        my_nullable_i32: &MyNullableI32(Some(10)),
        val: 1,
    };

    diesel::insert_into(my_entities::table)
        .values(&obj)
        .execute(&mut conn)
        .expect("Couldn't insert struct into my_entities");

    let found: Vec<MyEntity> = my_entities::table.load(&mut conn).unwrap();
    println!("found: {:?}", found);
    assert_eq!(found[0].id, *obj.id);
    assert_eq!(found[0].my_i32, obj.my_i32);
    assert_eq!(found[0].my_nullable_string, *obj.my_nullable_string);
    assert_eq!(found[0].my_nullable_i32, *obj.my_nullable_i32);
    assert_eq!(found[0].val, obj.val);
}

#[test]
fn does_roundtrip_nulls() {
    let mut conn = setup();
    let obj = MyEntity {
        id: MyIdString("WooHoo".into()),
        my_i32: MyI32(10),
        my_nullable_string: MyNullableString(None),
        my_nullable_i32: MyNullableI32(None),
        val: 1,
    };

    diesel::insert_into(my_entities::table)
        .values(&obj)
        .execute(&mut conn)
        .expect("Couldn't insert struct into my_entities");

    let found: Vec<MyEntity> = my_entities::table.load(&mut conn).unwrap();
    println!("found: {:?}", found);
    assert_eq!(found[0], obj);
}

#[test]
fn queryable() {
    let mut conn = setup();
    let objs = vec![
        MyEntity {
            id: MyIdString("WooHoo".into()),
            my_i32: MyI32(10),
            my_nullable_string: MyNullableString(Some("WooHoo".into())),
            my_nullable_i32: MyNullableI32(Some(10)),
            val: 1,
        },
        MyEntity {
            id: MyIdString("boo".into()),
            my_i32: MyI32(20),
            my_nullable_string: MyNullableString(None),
            my_nullable_i32: MyNullableI32(None),
            val: 2,
        },
    ];

    diesel::insert_into(my_entities::table)
        .values(&objs)
        .execute(&mut conn)
        .expect("Couldn't insert struct into my_entities");

    let ids: Vec<MyIdString> = my_entities::table
        .select(my_entities::columns::id)
        .load(&mut conn)
        .unwrap();
    assert_eq!(&ids[0], &objs[0].id);
    assert_eq!(&ids[1], &objs[1].id);
}

#[test]
fn query_as_id() {
    let mut conn = setup();
    let expected = MyEntity {
        id: MyIdString("WooHoo".into()),
        my_i32: MyI32(10),
        my_nullable_string: MyNullableString(Some("WooHoo".into())),
        my_nullable_i32: MyNullableI32(Some(10)),
        val: 1,
    };
    let objs = vec![
        MyEntity {
            id: MyIdString("loop".into()),
            my_i32: MyI32(0),
            my_nullable_string: MyNullableString(Some("loop".into())),
            my_nullable_i32: MyNullableI32(Some(0)),
            val: 0,
        },
        expected.clone(),
        MyEntity {
            id: MyIdString("boo".into()),
            my_i32: MyI32(20),
            my_nullable_string: MyNullableString(None),
            my_nullable_i32: MyNullableI32(None),
            val: 2,
        },
    ];

    diesel::insert_into(my_entities::table)
        .values(&objs)
        .execute(&mut conn)
        .expect("Couldn't insert struct into my_entities");

    let ids: Vec<MyEntity> = my_entities::table
        .filter(my_entities::id.eq(MyIdString("WooHoo".into())))
        .load(&mut conn)
        .unwrap();
    assert_eq!(ids, vec![expected])
}

#[test]
fn query_as_underlying_type() {
    let mut conn = setup();
    let expected = MyEntity {
        id: MyIdString("WooHoo".into()),
        my_i32: MyI32(10),
        my_nullable_string: MyNullableString(Some("WooHoo".into())),
        my_nullable_i32: MyNullableI32(Some(10)),
        val: 1,
    };
    let objs = vec![
        MyEntity {
            my_i32: MyI32(0),
            id: MyIdString("loop".into()),
            my_nullable_string: MyNullableString(Some("loop".into())),
            my_nullable_i32: MyNullableI32(Some(0)),
            val: 0,
        },
        expected.clone(),
        MyEntity {
            id: MyIdString("boo".into()),
            my_i32: MyI32(20),
            my_nullable_string: MyNullableString(None),
            my_nullable_i32: MyNullableI32(None),
            val: 2,
        },
    ];

    diesel::insert_into(my_entities::table)
        .values(&objs)
        .execute(&mut conn)
        .expect("Couldn't insert struct into my_entities");

    let ids: Vec<MyEntity> = my_entities::table
        .filter(my_entities::id.eq("WooHoo".to_string()))
        .load(&mut conn)
        .unwrap();
    assert_eq!(ids, vec![expected])
}

#[test]
fn set() {
    let mut conn = setup();
    let expected = MyEntity {
        id: MyIdString("WooHoo".into()),
        my_i32: MyI32(10),
        my_nullable_string: MyNullableString(Some("WooHoo".into())),
        my_nullable_i32: MyNullableI32(Some(10)),
        val: 1,
    };
    let objs = vec![
        MyEntity {
            id: MyIdString("loop".into()),
            my_i32: MyI32(0),
            my_nullable_string: MyNullableString(Some("loop".into())),
            my_nullable_i32: MyNullableI32(Some(0)),
            val: 0,
        },
        expected.clone(),
        MyEntity {
            id: MyIdString("boo".into()),
            my_i32: MyI32(20),
            my_nullable_string: MyNullableString(None),
            my_nullable_i32: MyNullableI32(None),
            val: 2,
        },
    ];

    diesel::insert_into(my_entities::table)
        .values(&objs)
        .execute(&mut conn)
        .expect("Couldn't insert struct into my_entities");

    let new_id = MyIdString("Oh My".into());
    diesel::update(my_entities::table.find(&expected.id))
        .set(my_entities::id.eq(&new_id))
        .execute(&mut conn)
        .unwrap();
    let updated_ids: Vec<MyEntity> = my_entities::table
        .filter(my_entities::id.eq(&new_id))
        .load(&mut conn)
        .unwrap();
    assert_eq!(
        updated_ids,
        vec![MyEntity {
            id: new_id,
            my_i32: MyI32(10),
            my_nullable_string: MyNullableString(Some("WooHoo".into())),
            my_nullable_i32: MyNullableI32(Some(10)),
            val: 1
        }]
    )
}
