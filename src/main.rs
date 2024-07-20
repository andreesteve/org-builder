use std::collections::{HashMap, HashSet};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Person {
    #[serde(alias = "id")]
    id: u64,

    #[serde(alias = "manager id")]
    manager_id: Option<u64>,

    #[serde(alias = "cost center")]
    #[serde(default)]
    cost_center: String,

    #[serde(alias = "job title")]
    #[serde(default)]
    job_title: String,
}

struct Position {
    person: Person,
}

fn main() {
    let mut people  = HashMap::new();
    let mut roots = Vec::new();

    // add all people
    let data: Vec<Person> = load_csv("people.csv").expect("Couldn't parse");
    for person in data {
        if person.manager_id.is_none() {
            roots.push(person.id);
        }
        people.insert(person.id, Position{ person });
    }

    // there could be managers whose managers are not in the list
    // track those as roots
    for (_, pos) in people.iter() {
        let person = &pos.person;
        if let Some(manager_id) = person.manager_id {
            if !people.contains_key(&manager_id) {
                println!("Person {} has manager {} that cannot be found in the list. Proceeding as if this person had no manager.",
                    person.id, manager_id);
                roots.push(person.id);
            }
        }
    }

    // make sure roots have no managers
    for id in roots.iter() {
        people.get_mut(id).unwrap().person.manager_id = None;
    }

    // identify cycles
    {
        let mut seen = HashSet::new();
        let mut cycle = Vec::new();
        for (_, pos) in people.iter() {
            let mut person = &pos.person;
            if seen.contains(&person.id) {
                continue;
            }

            cycle.clear();
            loop {
                let id = person.id;
                cycle.push(id);
                if !seen.contains(&id) {
                    seen.insert(id);
                    if let Some(manager_id) = person.manager_id {
                        person = &people.get(&manager_id).unwrap().person;
                    } else {
                        // no manager, stop
                        break;
                    }
                } else {
                    println!("Detected a cycle between employees: {:#?}.",
                        cycle);
                    panic!("Cannot continue do due cycle identified.")
                }
            }
        }
    }
}

fn load_csv<'a, T : DeserializeOwned>(path: &str) -> Option<Vec<T>> {
    if std::path::Path::new(path).exists() {
        let mut s = Vec::new();
        let mut reader = csv::Reader::from_path(path).expect("File exists but couldn't deserialize it.");
        for result in reader.deserialize() {
            let record: T = result.expect("Record exists but couldn't deserialize it.");
            s.push(record);
        }

        Some(s)
    } else {
        None
    }
}