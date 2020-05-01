use std::collections::HashMap;
use std::collections::HashSet;
use key_vec::KeyVec;
use std::error::Error;
use std::result::Result;

pub enum KeyValueItem {
    Atomic(i32),
    Scalar(Vec<u8>),
    List(Vec<Vec<u8>>),
    Set(HashSet<Vec<u8>>),
    SortedVec(KeyVec<i32, Vec<u8>>),
}

pub struct KeyValueStore {
    items: HashMap<String, KeyValueItem>,
}

impl KeyValueStore {
    pub fn new() -> Self {
        KeyValueStore {
            items: HashMap::new(),
        }
    }

    pub fn incr(&mut self, key: &str, value: i32) -> Result<i32, Box<dyn Error>> {
        let mut orig = 0;
        self.items
            .entry(key.to_string())
            .and_modify(|v| {
                if let KeyValueItem::Atomic(ref x) = v {
                    orig = *x;
                    *v = KeyValueItem::Atomic(x + value);
                }
            })
            .or_insert(KeyValueItem::Atomic(value));
        Ok(orig + value)
    }

    pub fn del(&mut self, key: &str) -> Result<(), Box<dyn Error>> {
        self.items.remove(key);
        Ok(())
    }

    pub fn exists(&self, key: &str) -> Result<bool, Box<dyn Error>> {
        Ok(self.items.contains_key(key))
    }

    pub fn get(&self, key: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        self.items.get(key).map_or_else(
            || Err("No such key".into()),
            |v| {
                if let KeyValueItem::Scalar(ref s) = v {
                    Ok(s.clone())
                } else {
                    Err("Attempt to fetch non-scalar".into())
                }
            },
        )
    }

    pub fn lrange(&self, key: &str, start: i32, stop: i32) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
        let start = start.max(0);
        self.items.get(key).map_or_else(
            || Ok(vec![vec![]]),
            |v| {
                if let KeyValueItem::List(l) = v {
                    let stop = stop.min(l.len() as _);
                    Ok(l.as_slice()[start as _..stop as _].to_vec())
                } else {
                    Err("Attempt to fetch non-list".into())
                }
            },
        )
    }

    pub fn lpush(&mut self, key: &str, value: Vec<u8>) -> Result<i32, Box<dyn Error>> {
        let mut len = 1;
        self.items
            .entry(key.to_string())
            .and_modify(|v| {
                if let KeyValueItem::List(ref l) = v {
                    let mut list = Vec::new();
                    list.extend_from_slice(&l);
                    list.push(value.clone());
                    len = list.len();
                    *v = KeyValueItem::List(list);
                }
            })
            .or_insert_with(|| KeyValueItem::List(vec![value]));
        Ok(len as _)
    }

    pub fn sv_insert(&mut self, key:&str, value: &(i32, Vec<u8>), overwrite: bool)-> Result<bool, Box<dyn Error>> {
        let mut result = false;
        self.items
            .entry(key.to_string())
            .and_modify(|v| {
                if let KeyValueItem::SortedVec(ref mut kvec) = v {
                    if let Some(_current_existing_value) = kvec.get(&value.0){
                        if overwrite {
                            kvec.insert(value.0, value.1.clone()); 
                            result = true;
                        }
                        else{
                            result = false;
                        }
                    }
                    else{
                        kvec.insert(value.0, value.1.clone());
                        result = true;
                    }
                }
            })
            .or_insert_with(|| {
                let mut kvec = KeyVec::new();
                kvec.insert(value.0, value.1.clone());
                result = true;
                KeyValueItem::SortedVec(kvec)
            });
        Ok(result)
    }

    pub fn sv_into_vec(&self, key: &str) -> Result<Vec<(i32, Vec<u8>)>, Box<dyn Error>> {
        match self.items.get(key){
            None=>Ok(Vec::new()),
            Some(v)=>{
                if let KeyValueItem::SortedVec(ref kvec) = v {
                    Ok(kvec.clone().into_vec())
                }
                else{
                    return Err("Attemp to call to a non Sorted Vec".into());
                }
            }
        }
    }

    pub fn sv_tail_off(&mut self, key: &str, remain: usize) -> Result<usize, Box<dyn Error>>{
        let mut len = 0;
        self.items.entry(key.to_string()).and_modify(|v| {
            if let KeyValueItem::SortedVec(ref mut kvec) = v {
                len = kvec.len();
                println!("kvec len remain: {},{}", len, remain);
                if len > remain{
                
                    let mut i: usize = len;
                    loop{
                        if i == remain {
                            break;
                        }
                        println!("inside loop: kvec len and i: {},{}", len, i); 
                        kvec.remove_index(i - 1);
                        i = i - 1;
                    }
                    
                }
                len = kvec.len();
            }
            
        });
        
        Ok(len)
    }
    pub fn set(&mut self, key: &str, value: Vec<u8>) -> Result<(), Box<dyn Error>> {
        self.items
            .entry(key.to_string())
            .and_modify(|v| {
                if let KeyValueItem::Scalar(_) = v {
                    *v = KeyValueItem::Scalar(value.clone());
                }
            })
            .or_insert(KeyValueItem::Scalar(value));
        Ok(())
    }

    pub fn lrem(&mut self, key: &str, value: Vec<u8>) -> Result<i32, Box<dyn Error>> {
        let mut len: i32 = 0;
        self.items.entry(key.to_string()).and_modify(|v| {
            if let KeyValueItem::List(ref l) = v {
                let list: Vec<Vec<u8>> = l
                    .iter()
                    .filter(|i| **i != value)
                    .map(|v| v.clone())
                    .collect();
                len = list.len() as _;
                *v = KeyValueItem::List(list);
            }
        });
        Ok(len)
    }

    pub fn sadd(&mut self, key: &str, value: Vec<u8>) -> Result<i32, Box<dyn Error>> {
        let mut len: i32 = 1;
        self.items
            .entry(key.to_string())
            .and_modify(|v| {
                if let KeyValueItem::Set(ref mut s) = v {
                    s.insert(value.clone());
                    len = s.len() as _;
                }
            })
            .or_insert_with(|| new_set(value));
        Ok(len)
    }

    pub fn srem(&mut self, key: &str, value: Vec<u8>) -> Result<i32, Box<dyn Error>> {
        let mut len: i32 = 0;
        self.items
            .entry(key.to_string())
            .and_modify(|v| {
                if let KeyValueItem::Set(ref mut s) = v {
                    s.remove(&value);
                    len = s.len() as _;
                }
            })
            .or_insert_with(|| KeyValueItem::Set(HashSet::new()));
        Ok(len)
    }

    pub fn sunion(&self, keys: Vec<String>) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
        let union = self
            .items
            .iter()
            .filter_map(|(k, v)| {
                if keys.contains(k) {
                    if let KeyValueItem::Set(s) = v {
                        Some(s.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .fold(HashSet::new(), |acc, x| acc.union(&x).cloned().collect());

        Ok(union.iter().cloned().collect())
    }

    pub fn sinter(&self, keys: Vec<String>) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
        let sets: Vec<HashSet<Vec<u8>>> = self
            .items
            .iter()
            .filter_map(|(k, v)| {
                if keys.contains(k) {
                    if let KeyValueItem::Set(s) = v {
                        Some(s.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        let set1 = &sets[0];
        let inter = set1
            .iter()
            .filter(|k| sets.as_slice().iter().all(|s| s.contains(*k)));
        Ok(inter.cloned().collect())
    }

    pub fn smembers(&self, key: String) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
        self.items.get(&key).map_or_else(
            || Ok(vec![]),
            |v| {
                if let KeyValueItem::Set(ref s) = v {
                    Ok(s.iter().cloned().collect())
                } else {
                    Err("attempt to query non-set".into())
                }
            },
        )
    }
}

fn new_set(value: Vec<u8>) -> KeyValueItem {
    let mut x = HashSet::new();
    x.insert(value);
    KeyValueItem::Set(x)
}

#[cfg(test)]
mod test {
    use super::KeyValueStore;

    fn gen_store() -> KeyValueStore {
        let mut store = KeyValueStore::new();
        store.sadd("test", "bob".to_owned().into_bytes()).unwrap();
        store.sadd("test", "alice".to_owned().into_bytes()).unwrap();
        store.sadd("test", "dave".to_owned().into_bytes()).unwrap();
        store.sadd("test2", "bob".to_owned().into_bytes()).unwrap();
        store.sadd("test2", "dave".to_owned().into_bytes()).unwrap();

        store.lpush("list1", "first".to_owned().into_bytes()).unwrap();
        store.lpush("list1", "second".to_owned().into_bytes()).unwrap();
        store.lpush("list1", "third".to_owned().into_bytes()).unwrap();

        store.incr("counter", 5).unwrap();

        store.set("setkey", "setval".to_owned().into_bytes()).unwrap();

        store
    }

    #[test]
    fn test_intersect() {
        let store = gen_store();

        let inter = store
            .sinter(vec!["test".to_string(), "test2".to_string()])
            .unwrap();
        assert!(inter.contains(&"bob".to_owned().into_bytes()));
        assert!(inter.contains(&"dave".to_owned().into_bytes()));
        assert_eq!(false, inter.contains(&"alice".to_owned().into_bytes()));
    }

    #[test]
    fn test_union() {
        let store = gen_store();

        let union = store
            .sunion(vec!["test".to_string(), "test2".to_string()])
            .unwrap();
        assert_eq!(3, union.len());
    }

    #[test]
    fn test_get_set() {
        let store = gen_store();

        assert_eq!("setval".to_owned().into_bytes(), store.get("setkey").unwrap());
    }

    #[test]
    fn test_list() {
        let store = gen_store();
        assert_eq!(
            vec!["first".to_owned().into_bytes(), "second".to_owned().into_bytes(), "third".to_owned().into_bytes()],
            store.lrange("list1", 0, 100).unwrap()
        );
    }

    #[test]
    fn test_incr() {
        let mut store = gen_store();

        let a = store.incr("counter", 1).unwrap();
        let b = store.incr("counter", 1).unwrap();
        let c = store.incr("counter", -3).unwrap();

        assert_eq!(a, 6);
        assert_eq!(b, 7);
        assert_eq!(c, 4);
    }

    #[test]
    fn test_exists_and_del() {
        let mut store = gen_store();

        store.set("thenumber", "42".to_owned().into_bytes());
        assert!(store.exists("thenumber").unwrap());
        store.del("thenumber").unwrap();
        assert_eq!(false, store.exists("thenumber").unwrap());
        store.set("thenumber", "41".to_owned().into_bytes());
        assert!(store.exists("thenumber").unwrap());
    }
    #[test]
    fn test_sorted_vec(){
        let mut store = gen_store();
        let tup0 = (0, "zero".to_owned().into_bytes());
        let tup1 = (1, "one".to_owned().into_bytes());
        let tup3 = (3, "three".to_owned().into_bytes());
        store.sv_insert("sorted", &tup1, false);
        store.sv_insert("sorted", &tup0, false);
        store.sv_insert("sorted", &tup3, false);
        let r = store.sv_into_vec("sorted").unwrap();
        assert_eq!(r, vec![tup0.clone(),tup1.clone(),tup3.clone()]);
        let tup2 =(2, "two".to_owned().into_bytes()); 
        store.sv_insert("sorted", &tup2, false);
        let r = store.sv_into_vec("sorted").unwrap();
        assert_eq!(r, vec![tup0.clone(),tup1.clone(),tup2.clone(),tup3.clone()]);
        store.sv_tail_off("sorted", 2);
        let r = store.sv_into_vec("sorted").unwrap();
        assert_eq!(r, vec![tup0.clone(),tup1.clone()]);
    }
}