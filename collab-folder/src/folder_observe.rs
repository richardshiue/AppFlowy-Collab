use dashmap::DashMap;
use std::sync::Arc;

use collab::core::collab::{IndexContent, IndexContentSender};
use collab::preclude::{
  DeepObservable, EntryChange, Event, MapRef, Subscription, ToJson, YrsValue,
};
use serde_json::json;
use tokio::sync::broadcast;

use crate::section::SectionMap;
use crate::{ParentChildRelations, UserId, View, ViewIndexContent, view_from_map_ref};

#[derive(Debug, Clone)]
pub enum ViewChange {
  DidCreateView { view: View },
  DidDeleteView { views: Vec<Arc<View>> },
  DidUpdate { view: View },
}

pub type ViewChangeSender = broadcast::Sender<ViewChange>;
pub type ViewChangeReceiver = broadcast::Receiver<ViewChange>;

pub(crate) fn subscribe_folder_change(root: &mut MapRef) -> Subscription {
  root.observe_deep(move |txn, events| {
    for deep_event in events.iter() {
      match deep_event {
        Event::Text(_) => {},
        Event::Array(_) => {},
        Event::Map(event) => {
          for c in event.keys(txn).values() {
            match c {
              EntryChange::Inserted(v) => {
                if let YrsValue::YMap(map_ref) = v {
                  tracing::trace!("folder change: Inserted: {}", map_ref.to_json(txn));
                }
              },
              EntryChange::Updated(_k, v) => {
                if let YrsValue::YMap(map_ref) = v {
                  tracing::trace!("folder change: Updated: {}", map_ref.to_json(txn));
                }
              },
              EntryChange::Removed(v) => if let YrsValue::YMap(_map_ref) = v {},
            }
          }
        },
        Event::XmlFragment(_) => {},
        Event::XmlText(_) => {},
        #[allow(unreachable_patterns)]
        _ => {},
      }
    }
  })
}

pub(crate) fn subscribe_view_change(
  _uid: &UserId,
  root: &mut MapRef,
  view_cache: Arc<DashMap<String, Arc<View>>>,
  change_tx: ViewChangeSender,
  view_relations: Arc<ParentChildRelations>,
  section_map: Arc<SectionMap>,
  index_sender: IndexContentSender,
) -> Subscription {
  root.observe_deep(move |txn, events| {
    for deep_event in events.iter() {
      match deep_event {
        Event::Text(_) => {},
        Event::Array(_) => {},
        Event::Map(event) => {
          for c in event.keys(txn).values() {
            let change_tx = change_tx.clone();
            match c {
              EntryChange::Inserted(v) => {
                if let YrsValue::YMap(map_ref) = v {
                  if let Some(view) = view_from_map_ref(map_ref, txn, &view_relations, &section_map)
                  {
                    view_cache.insert(view.id.clone(), Arc::new(view.clone()));

                    // Send indexing view
                    let index_content = ViewIndexContent::from(&view);
                    let _ = index_sender.send(IndexContent::Create(json!(index_content)));

                    let _ = change_tx.send(ViewChange::DidCreateView { view });
                  }
                }
              },
              EntryChange::Updated(_, _) => {
                if let Some(view) =
                  view_from_map_ref(event.target(), txn, &view_relations, &section_map)
                {
                  view_cache.insert(view.id.clone(), Arc::new(view.clone()));

                  // Update indexing view
                  let index_content = ViewIndexContent::from(&view);
                  let _ = index_sender.send(IndexContent::Update(json!(index_content)));

                  let _ = change_tx.send(ViewChange::DidUpdate { view });
                }
              },
              EntryChange::Removed(_) => {
                let views: Vec<_> = event
                  .keys(txn)
                  .iter()
                  .flat_map(|(k, _)| view_cache.remove(&**k).map(|v| v.1))
                  .collect();

                if !views.is_empty() {
                  // Delete indexing views
                  let delete_ids: Vec<String> = views.iter().map(|v| v.id.to_owned()).collect();
                  let _ = index_sender.send(IndexContent::Delete(delete_ids));

                  let _ = change_tx.send(ViewChange::DidDeleteView { views });
                }
              },
            }
          }
        },
        Event::XmlFragment(_) => {},
        Event::XmlText(_) => {},
        #[allow(unreachable_patterns)]
        _ => {},
      }
    }
  })
}
