use crate::helper::{create_database_with_db, restore_database_from_db, DatabaseTest};
use assert_json_diff::assert_json_eq;

use collab_database::block::CreateRowParams;
use collab_persistence::CollabKV;
use serde_json::{json, Value};
use std::sync::Arc;

#[test]
fn restore_row_from_disk_test() {
  let (db, database_test) = create_database_with_db(1, "1");
  let row_1 = CreateRowParams {
    id: 1.into(),
    ..Default::default()
  };
  let row_2 = CreateRowParams {
    id: 2.into(),
    ..Default::default()
  };
  database_test.push_row(row_1.clone());
  database_test.push_row(row_2.clone());
  drop(database_test);

  let database_test = restore_database_from_db(1, "1", db);
  let rows = database_test.get_rows_for_view("v1");
  assert_eq!(rows.len(), 2);

  assert!(rows.iter().any(|row| row.id == row_1.id));
  assert!(rows.iter().any(|row| row.id == row_2.id));
}

#[test]
fn restore_from_disk_test() {
  let (db, database_test, expected) = create_database_with_view();
  assert_json_eq!(expected, database_test.to_json_value());

  // Restore from disk
  let database_test = restore_database_from_db(1, "1", db);
  assert_json_eq!(expected, database_test.to_json_value());
}

#[test]
fn restore_from_disk_with_different_database_id_test() {
  let (db, _, _) = create_database_with_view();
  let database_test = restore_database_from_db(1, "1", db);
  assert_json_eq!(
    json!( {
      "fields": [],
      "rows": [],
      "views": [
        {
          "created_at": 0,
          "database_id": "1",
          "field_orders": [],
          "filters": [],
          "group_settings": [],
          "id": "v1",
          "layout": 0,
          "layout_settings": {},
          "modified_at": 0,
          "name": "my first grid",
          "row_orders": [],
          "sorts": []
        }
      ]
    }),
    database_test.to_json_value()
  );
}

#[test]
fn restore_from_disk_with_different_uid_test() {
  let (db, _, _) = create_database_with_view();
  let database_test = restore_database_from_db(1, "1", db);
  assert_json_eq!(
    json!( {
      "fields": [],
      "rows": [],
      "views": [
        {
          "created_at": 0,
          "database_id": "1",
          "field_orders": [],
          "filters": [],
          "group_settings": [],
          "id": "v1",
          "layout": 0,
          "layout_settings": {},
          "modified_at": 0,
          "name": "my first grid",
          "row_orders": [],
          "sorts": []
        }
      ]
    }),
    database_test.to_json_value()
  );
}

fn create_database_with_view() -> (Arc<CollabKV>, DatabaseTest, Value) {
  let (db, database_test) = create_database_with_db(1, "1");
  let expected = json!({
    "fields": [],
    "rows": [],
    "views": [
      {
        "created_at": 0,
        "database_id": "1",
        "field_orders": [],
        "filters": [],
        "group_settings": [],
        "id": "v1",
        "layout": 0,
        "layout_settings": {},
        "modified_at": 0,
        "name": "my first grid",
        "row_orders": [],
        "sorts": []
      }
    ]
  });
  (db, database_test, expected)
}