use crate::database::gen_database_file_id;
use crate::entity::FieldType;
use crate::fields::{StringifyTypeOption, TypeOptionData, TypeOptionDataBuilder};
use crate::rows::{new_cell_builder, Cell};
use crate::template::entity::CELL_DATA;
use collab::util::AnyMapExt;
use serde::{Deserialize, Deserializer, Serialize};
use serde_repr::Serialize_repr;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::sync::Arc;
use yrs::Any;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaTypeOption {
  pub hide_file_names: bool,
}

impl Default for MediaTypeOption {
  fn default() -> Self {
    Self {
      hide_file_names: true,
    }
  }
}

impl StringifyTypeOption for MediaTypeOption {
  fn stringify_cell(&self, cell: &Cell) -> String {
    match cell.get_as::<MediaCellData>(CELL_DATA) {
      None => "".to_string(),
      Some(s) => s.to_string(),
    }
  }

  fn stringify_text(&self, text: &str) -> String {
    let data = MediaCellData::from(text.to_string());
    data
      .files
      .into_iter()
      .map(|file| file.name)
      .collect::<Vec<_>>()
      .join(", ")
  }
}

impl From<TypeOptionData> for MediaTypeOption {
  fn from(data: TypeOptionData) -> Self {
    data
      .get_as::<String>("content")
      .map(|s| serde_json::from_str::<MediaTypeOption>(&s).unwrap_or_default())
      .unwrap_or_default()
  }
}

impl From<MediaTypeOption> for TypeOptionData {
  fn from(data: MediaTypeOption) -> Self {
    let content = serde_json::to_string(&data).unwrap_or_default();
    TypeOptionDataBuilder::from([("content".into(), content.into())])
  }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct MediaCellData {
  pub files: Vec<MediaFile>,
}

impl From<MediaCellData> for Any {
  fn from(data: MediaCellData) -> Self {
    Any::Array(Arc::from(
      data
        .files
        .clone()
        .into_iter()
        .map(|file| Any::String(Arc::from(serde_json::to_string(&file).unwrap_or_default())))
        .collect::<Vec<_>>(),
    ))
  }
}

impl TryFrom<Any> for MediaCellData {
  type Error = Any;

  fn try_from(value: Any) -> Result<Self, Self::Error> {
    match value {
      Any::Array(array) => {
        let files = array
          .iter()
          .flat_map(|item| {
            if let Any::String(string) = item {
              Some(serde_json::from_str::<MediaFile>(string).unwrap_or_default())
            } else {
              None
            }
          })
          .collect();
        Ok(Self { files })
      },
      _ => Ok(Self::default()),
    }
  }
}
impl From<&Cell> for MediaCellData {
  fn from(cell: &Cell) -> Self {
    cell.get_as::<MediaCellData>(CELL_DATA).unwrap_or_default()
  }
}

impl From<MediaCellData> for Cell {
  fn from(value: MediaCellData) -> Self {
    let mut cell = new_cell_builder(FieldType::Media);
    cell.insert(CELL_DATA.into(), value.into());
    cell
  }
}

impl From<String> for MediaCellData {
  fn from(s: String) -> Self {
    if s.is_empty() {
      return MediaCellData { files: vec![] };
    }

    let files = s
      .split(", ")
      .map(|file: &str| serde_json::from_str::<MediaFile>(file).unwrap_or_default())
      .collect::<Vec<_>>();

    MediaCellData { files }
  }
}

impl ToString for MediaCellData {
  fn to_string(&self) -> String {
    self
      .files
      .iter()
      .map(|file| file.name.clone())
      .collect::<Vec<_>>()
      .join(", ")
  }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaFile {
  pub id: String,
  pub name: String,
  pub url: String,
  pub upload_type: MediaUploadType,
  pub file_type: MediaFileType,
}

impl MediaFile {
  pub fn new(
    name: String,
    url: String,
    upload_type: MediaUploadType,
    file_type: MediaFileType,
  ) -> Self {
    Self {
      id: gen_database_file_id(),
      name,
      url,
      upload_type,
      file_type,
    }
  }

  pub fn rename(&self, new_name: String) -> Self {
    Self {
      id: self.id.clone(),
      name: new_name,
      url: self.url.clone(),
      upload_type: self.upload_type.clone(),
      file_type: self.file_type.clone(),
    }
  }
}

impl Display for MediaFile {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "MediaFile(id: {}, name: {}, url: {}, upload_type: {:?}, file_type: {:?})",
      self.id, self.name, self.url, self.upload_type, self.file_type
    )
  }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize_repr)]
#[repr(u8)]
pub enum MediaFileType {
  #[default]
  Other = 0,
  Image = 1,
  Link = 2,
  Document = 3,
  Archive = 4,
  Video = 5,
  Audio = 6,
  Text = 7,
}

impl MediaFileType {
  pub fn from_file<T: AsRef<Path>>(path: T) -> MediaFileType {
    match path
      .as_ref()
      .extension()
      .and_then(std::ffi::OsStr::to_str)
      .unwrap_or("")
      .to_lowercase()
      .as_str()
    {
      "jpg" | "jpeg" | "png" | "gif" => MediaFileType::Image,
      "zip" | "rar" | "tar" => MediaFileType::Archive,
      "mp4" | "mov" | "avi" => MediaFileType::Video,
      "mp3" | "wav" => MediaFileType::Audio,
      "txt" => MediaFileType::Text,
      "doc" | "docx" => MediaFileType::Document,
      "html" | "htm" => MediaFileType::Link,
      _ => MediaFileType::Other,
    }
  }
}
impl<'de> Deserialize<'de> for MediaFileType {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    struct MediaFileTypeVisitor;

    impl<'de> serde::de::Visitor<'de> for MediaFileTypeVisitor {
      type Value = MediaFileType;

      fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string or a u8 representing MediaFileType")
      }

      fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
      where
        E: serde::de::Error,
      {
        match value {
          0 => Ok(MediaFileType::Other),
          1 => Ok(MediaFileType::Image),
          2 => Ok(MediaFileType::Link),
          3 => Ok(MediaFileType::Document),
          4 => Ok(MediaFileType::Archive),
          5 => Ok(MediaFileType::Video),
          6 => Ok(MediaFileType::Audio),
          7 => Ok(MediaFileType::Text),
          _ => Err(E::custom(format!(
            "Unknown numeric value for MediaFileType: {}",
            value
          ))),
        }
      }

      fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
      where
        E: serde::de::Error,
      {
        match value {
          "Other" => Ok(MediaFileType::Other),
          "Image" => Ok(MediaFileType::Image),
          "Link" => Ok(MediaFileType::Link),
          "Document" => Ok(MediaFileType::Document),
          "Archive" => Ok(MediaFileType::Archive),
          "Video" => Ok(MediaFileType::Video),
          "Audio" => Ok(MediaFileType::Audio),
          "Text" => Ok(MediaFileType::Text),
          _ => Err(E::custom(format!(
            "Unknown string variant for MediaFileType: {}",
            value
          ))),
        }
      }
    }

    deserializer.deserialize_any(MediaFileTypeVisitor)
  }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize_repr)]
#[repr(u8)]
pub enum MediaUploadType {
  #[default]
  Local = 0,
  /// Network means file is external http URL
  Network = 1,
  /// Cloud means file stored in appflowy cloud
  Cloud = 2,
}

impl<'de> Deserialize<'de> for MediaUploadType {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    struct MediaUploadTypeVisitor;

    impl<'de> serde::de::Visitor<'de> for MediaUploadTypeVisitor {
      type Value = MediaUploadType;

      fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string or a u8 representing MediaUploadType")
      }

      fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
      where
        E: serde::de::Error,
      {
        match value {
          0 => Ok(MediaUploadType::Local),
          1 => Ok(MediaUploadType::Network),
          2 => Ok(MediaUploadType::Cloud),
          _ => Err(E::custom(format!(
            "Unknown numeric value for MediaUploadType: {}",
            value
          ))),
        }
      }

      fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
      where
        E: serde::de::Error,
      {
        match value {
          "Local" | "LocalMedia" => Ok(MediaUploadType::Local),
          "Network" | "NetworkMedia" => Ok(MediaUploadType::Network),
          "Cloud" | "CloudMedia" => Ok(MediaUploadType::Cloud),
          _ => Err(E::custom(format!(
            "Unknown string variant for MediaUploadType: {}",
            value
          ))),
        }
      }
    }

    deserializer.deserialize_any(MediaUploadTypeVisitor)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json;

  #[test]
  fn test_serialize_deserialize_media_file() {
    let media_file = MediaFile {
      id: "123".to_string(),
      name: "test_file".to_string(),
      url: "http://example.com/file".to_string(),
      upload_type: MediaUploadType::Cloud,
      file_type: MediaFileType::Image,
    };

    // Serialize the MediaFile to a JSON string
    let serialized = serde_json::to_string(&media_file).unwrap();
    println!("Serialized MediaFile: {}", serialized);
    let deserialized: MediaFile = serde_json::from_str(&serialized).unwrap();
    assert_eq!(media_file, deserialized);
  }
}
