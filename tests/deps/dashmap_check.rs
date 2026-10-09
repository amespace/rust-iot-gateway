

#[cfg(test)]
mod tests {
    use std::{fmt::format, sync::Arc, thread};

use dashmap::DashMap;

     
     #[test]
     fn concurrent_insert_get_1000_tasks () {
        let map = Arc::new(DashMap::with_capacity(1024));

        let mut handlers = Vec::new();

        for i in 0..1000 {
            let m = map.clone();
            handlers.push(thread::spawn(move || {
                m.insert(i, format!("v{i}"));
                assert_eq!(m.get(&i).map(|s| s.value().clone()), Some(format!("v{i}")));
            }));
        }
        for h in handlers {
            h.join().unwrap();
        }
     }
}