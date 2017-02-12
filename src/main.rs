extern crate csv;
extern crate rustc_serialize;
extern crate rand;

use rustc_serialize::json;
use std::collections::HashMap;
use rand::{thread_rng, Rng};

#[derive(Debug)]
enum CompareType {
    Equal,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

#[derive(Debug)]
struct Attribute {
    name: String,
    values: Vec<String>,
    compare_type: CompareType,
}

impl Attribute {
    fn map_value(&self, value: Option<String>) -> u16 {
        match value {
            Some(value) => {
                match self.values.iter().position(|x| x == &value) {
                    Some(index) => (index + 1) as u16,
                    None => 0, // panic!("{:?} not found in {:?}", value, self)
                }
            }
            None => 0,
        }
    }

    fn evaluate_values(&self, b: &u16, a: &u16) -> bool {
        match self.compare_type {
            CompareType::Equal => a == b,
            CompareType::GreaterThanOrEqual => a >= b,
            CompareType::LessThanOrEqual => a <= b,
        }
    }
}

#[derive(Debug)]
struct Product {
    id: u32,
    name: String,
    values: Vec<u16>,
}

#[derive(Clone, Debug)]
struct Match(u32, u32);

// A model could be a function which returns the score function.. Much more like our definition
trait Model {
    fn train(&mut self);
    fn score(&self, a: u32, b: u32) -> u32;
}

struct SimpleModel {
    attributes: Vec<Attribute>,
    products: HashMap<u32, Product>,
    matches: Vec<Match>,
    match_count: Vec<u32>,
    match_sum: u32,
}

impl SimpleModel {
    fn train(&mut self) {
        for &Match(original, new) in self.matches.iter() {
            let original = self.products.get(&original).unwrap();
            let new = self.products.get(&new).unwrap();

            for (i, ((a, b), attribute)) in
                original.values.iter().zip(&new.values).zip(&self.attributes).enumerate() {
                // println!("{}: {} {:?} {} => {}", attribute.name, a, attribute.compare_type, b, attribute.evaluate_values(a, b));
                if attribute.evaluate_values(a, b) {
                    self.match_count[i] += 1;
                    self.match_sum += 1;
                }
            }
        }
    }

    fn score(&self, a: u32, b: u32) -> u32 {
        let original = self.products.get(&a).unwrap();
        let new = self.products.get(&b).unwrap();
        let mut score = 0;

        for (i, ((a, b), attribute)) in
            original.values.iter().zip(&new.values).zip(&self.attributes).enumerate() {
            if attribute.evaluate_values(a, b) {
                score += self.match_count[i];
            }
        }

        score
    }
    
    fn find_all_matches(&self, original: u32) -> Vec<u32> {
        let mut matches: Vec<(u32, u32)> = Vec::new();

        for (&k, _) in self.products.iter() {
            let score = self.score(original, k);
            matches.push((score, k));
        }

        matches.sort_by(|&(a, _), &(b, _)| b.cmp(&a));

        matches.iter().map(|&(_, x)| x).collect()
    }
}

fn main() {
    let mut rdr =
        csv::Reader::from_file("./data/subset_attributes.csv").unwrap().has_headers(false);
    let mut attributes = Vec::new();
    for record in rdr.decode() {
        let (name, values, compare_type): (String, String, u32) = record.unwrap();
        let values: Vec<String> = values.split(',').map(|s| s.to_string()).collect();
        let compare_type = match compare_type {
            1 => CompareType::Equal,
            2 => CompareType::GreaterThanOrEqual,
            3 => CompareType::LessThanOrEqual,
            _ => panic!("Unknown compare type: {}", compare_type),
        };
        attributes.push(Attribute {
            name: name,
            values: values,
            compare_type: compare_type,
        })
    }

    let mut rdr = csv::Reader::from_file("./data/subset.csv").unwrap().has_headers(false);
    let mut products: HashMap<u32, Product> = HashMap::new();
    for record in rdr.decode() {
        let (id, name, values): (u32, String, String) = record.unwrap();
        let values: Vec<Option<String>> = json::decode(&values).unwrap();
        let values: Vec<_> = values.into_iter()
            .zip(&attributes)
            .map(|(x, attribute)| attribute.map_value(x))
            .collect();

        products.insert(id,
                        Product {
                            id: id,
                            name: name,
                            values: values,
                        });
    }

    let mut rdr = csv::Reader::from_file("./data/matches.csv").unwrap().has_headers(false);
    let mut matches = Vec::new();
    for record in rdr.decode() {
        let (original, new): (u32, u32) = record.unwrap();

        if products.contains_key(&original) && products.contains_key(&new) {
            matches.push(Match(original, new));
        }
    }

    thread_rng().shuffle(&mut matches);

    let (train, test) = matches.split_at(((matches.len() as f32) * 0.95) as usize);

    let attributes_len = attributes.len();
    let mut simple_model = SimpleModel {
        attributes: attributes,
        products: products,
        matches: train.to_vec(),
        match_count: vec![0; attributes_len],
        match_sum: 0,
    };
    simple_model.train();

    for &Match(original, new) in test.iter() {
        let position = simple_model.find_all_matches(original).iter().position(|&x| x == new).unwrap();
        print!("{}, ", position);
    }
}