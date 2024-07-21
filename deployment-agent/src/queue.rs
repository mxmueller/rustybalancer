use std::collections::VecDeque;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize, Debug)]
pub struct QueueItem {
     name: String,
}

pub type SharedQueue = Arc<Mutex<VecDeque<QueueItem>>>;

pub async fn build_queue() -> SharedQueue {
     let queue = VecDeque::new();
     let shared_queue = Arc::new(Mutex::new(queue));

     // Hard-coded JSON elements
     let json_elements = vec![
          json!({"name": "item1"}),
          json!({"name": "item2"}),
          json!({"name": "item3"}),
     ];

     // Locking the shared queue and adding elements to it
     {
          let mut queue = shared_queue.lock().await;
          for element in json_elements {
               let item: QueueItem = serde_json::from_value(element).expect("Failed to deserialize JSON");
               queue.push_back(item);
          }
     }

     shared_queue
}

pub async fn enqueue(queue: SharedQueue, item: QueueItem) {
     let mut queue = queue.lock().await;
     queue.push_back(item);
}

pub async fn dequeue(queue: SharedQueue) -> Option<QueueItem> {
     let mut queue = queue.lock().await;
     queue.pop_front()
}

pub async fn display(queue: SharedQueue) {
     let queue = queue.lock().await;
     for item in queue.iter() {
          println!("{:?}", item);
     }
}
