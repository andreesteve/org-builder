use std::collections::{HashMap, HashSet, VecDeque};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
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

    #[serde(alias = "recursive reports")]
    #[serde(default)]
    recursive_reports: u64,
}

#[derive(Default, Serialize, Clone)]
struct Position {
    person: Person,
    reports: Vec<u64>
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

        people.insert(person.id, Position{ person, ..Default::default() });
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
        for id in people.keys().into_iter().copied().collect::<Vec<u64>>() {
            if seen.contains(&id) {
                continue;
            }

            let pos = people.get(&id).unwrap();
            let mut person = &pos.person;
            cycle.clear();

            loop {
                let id = person.id;
                if !seen.contains(&id) {
                    cycle.push(id);
                    seen.insert(id);
                    if let Some(manager_id) = person.manager_id {
                        let manager_pos = people.get_mut(&manager_id).unwrap();
                        manager_pos.reports.push(id);
                        person = &manager_pos.person;
                    } else {
                        // no manager, stop
                        break;
                    }
                } else if cycle.contains(&id) {
                    println!("Detected a cycle between employees: {:#?}.",
                        cycle);
                    panic!("Cannot continue do due cycle identified.")
                } else {
                    break;
                }
            }
        }
    }

    // depth first search
    let mut nodes = VecDeque::from_iter(people.values()
        .filter(|p| p.reports.len() == 0)
        .map(|p| p.person.id));

    for n in nodes.iter() {
        let p = people.get_mut(n).unwrap();
        p.person.recursive_reports = p.reports.len() as u64;
    }

    while let Some(next) = nodes.pop_back() {
        if let Some(manager_id) = people.get(&next).unwrap().person.manager_id {
            people.get_mut(&manager_id).unwrap().person.recursive_reports += people.get(&next).unwrap().person.recursive_reports + 1;

            if !nodes.contains(&manager_id) {
                nodes.push_front(manager_id);
            }
        }
    }

    // print managers by reports
    let mut sorted = people.values().cloned().collect::<Vec<Position>>();
    sorted.sort_by_key(|p| p.person.recursive_reports);
    sorted.reverse();

    for p in sorted.iter() {
        println!("{} -> R: {}, D: {}, M: {:?}, Reports: {:?}", p.person.id, p.person.recursive_reports, p.reports.len(), p.person.manager_id, p.reports);
    }

    let sorted_people = sorted.iter().map(|p| p.person.clone()).collect::<Vec<Person>>();
    save_csv("sorted.csv", &sorted_people);
}

fn save_csv<T : Serialize>(path: &str, records : &[T]) {
    let mut writer = csv::Writer::from_path(path).expect("Failed to write file");
    for r in records {
        writer.serialize(&r).expect("Failed to serialize disk");
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