use crate::entity::FieldType;
use crate::rows::{new_cell_builder, Cell};
use crate::template::entity::CELL_DATA;
use crate::template::util::{ToCellString, TypeOptionCellData};
use collab::util::AnyMapExt;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct NumberCellData(pub String);

impl TypeOptionCellData for NumberCellData {
  fn is_cell_empty(&self) -> bool {
    self.0.is_empty()
  }
}

impl AsRef<str> for NumberCellData {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

impl From<&Cell> for NumberCellData {
  fn from(cell: &Cell) -> Self {
    Self(cell.get_as(CELL_DATA).unwrap_or_default())
  }
}

impl From<NumberCellData> for Cell {
  fn from(data: NumberCellData) -> Self {
    let mut cell = new_cell_builder(FieldType::Number);
    cell.insert(CELL_DATA.into(), data.0.into());
    cell
  }
}

impl std::convert::From<String> for NumberCellData {
  fn from(s: String) -> Self {
    Self(s)
  }
}

impl ToCellString for NumberCellData {
  fn to_cell_string(&self) -> String {
    self.0.clone()
  }
}