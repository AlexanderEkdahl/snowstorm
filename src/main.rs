extern crate csv;
extern crate rustc_serialize;
extern crate rand;

use rustc_serialize::json;
use std::collections::HashMap;
use rand::{Rng, SeedableRng, StdRng};
use std::cmp::Ordering;

#[derive(Debug)]
enum CompareType {
    Equal,
    GreaterThanOrEqual,
    LessThanOrEqual,
    IntervalMatch(u16, u16),
    LinearInterval(u16, u16),
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

    fn evaluate_values(&self, a: &u16, b: &u16) -> (bool, f32) {
        if *a == 0 && *b == 0 {
            return (false, 0.0);
        } else if *a == 0 || *b == 0 {
            return (true, 0.0);
        }

        (true,
         match self.compare_type {
             CompareType::Equal => if *b == *a { 1.0 } else { 0.0 },
             CompareType::GreaterThanOrEqual => if *b >= *a { 1.0 } else { 0.0 },
             CompareType::LessThanOrEqual => if *b <= *a { 1.0 } else { 0.0 },
             CompareType::IntervalMatch(up, down) => {
                 if *b <= *a + up && *b >= if down >= *a { 0 } else { *a - down } {
                     1.0
                 } else {
                     0.0
                 }
             }
             CompareType::LinearInterval(up, down) => {
                 if *b == *a {
                     1.0
                 } else if *b > *a && up > 0 {
                     if 1.0 - (*b - *a) as f32 * 0.5 / up as f32 > 0.0 {
                         1.0 - (*b - *a) as f32 * 0.5 / up as f32
                     } else {
                         0.0
                     }
                 } else if *a > *b && down > 0 {
                     if 1.0 - (*a - *b) as f32 * 0.5 / down as f32 > 0.0 {
                         1.0 - (*a - *b) as f32 * 0.5 / down as f32
                     } else {
                         0.0
                     }
                 } else {
                     0.0
                 }
             }  
         })
    }
}

fn checked_division(dividend: f32, divisor: f32) -> Option<f32> {
    if divisor == 0.0 {
        None
    } else {
        Some(dividend / divisor)
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

struct SimpleModel<'a> {
    attributes: &'a Vec<Attribute>,
    products: &'a HashMap<u32, Product>,
    matches: &'a Vec<Match>,
    brain: Vec<f32>,
}

impl<'a> SimpleModel<'a> {
    fn train(&mut self) {
        let mut temporary_brain: Vec<(u32, f32)> = vec![(0, 0.0); self.attributes.len()];

        for &Match(original, new) in self.matches.iter() {
            let original = self.products.get(&original).unwrap();
            let new = self.products.get(&new).unwrap();

            for (i, attribute) in self.attributes.iter().enumerate() {
                if let (true, x) = attribute.evaluate_values(&original.values[i], &new.values[i]) {
                    temporary_brain[i] = (temporary_brain[i].0 + 1, temporary_brain[i].1 + x)
                }
            }
        }

        self.brain = temporary_brain.iter().map(|&(x, y)| checked_division(y, x as f32).unwrap_or(0.0)).collect();
    }

    fn score(&self, a: &u32, b: &u32) -> f32 {
        let original = self.products.get(&a).unwrap();
        let new = self.products.get(&b).unwrap();
        let mut score: f32 = 0.0;

        for (i, attribute) in self.attributes.iter().enumerate() {
            score += self.brain[i] *
                     attribute.evaluate_values(&original.values[i], &new.values[i]).1
        }
        score
    }

    fn visualize_score(&self, a: &u32, b: &u32) -> f32 {
        let original = self.products.get(&a).unwrap();
        let new = self.products.get(&b).unwrap();
        let mut score: f32 = 0.0;

        println!("Comparing: {} with {}", &original.name, &new.name);

        for (i, attribute) in self.attributes.iter().enumerate() {
            if original.values[i] != 0 || new.values[i] != 0 {
                println!("{name:>40.*} - {} {:?} {} => {}",
                         40,
                         if new.values[i] > 0 {
                             &attribute.values[(new.values[i] - 1) as usize]
                         } else {
                             "n/a"
                         },
                         attribute.compare_type,
                         if original.values[i] > 0 {
                             &attribute.values[(original.values[i] - 1) as usize]
                         } else {
                             "n/a"
                         },
                         self.brain[i] *
                         attribute.evaluate_values(&original.values[i], &new.values[i]).1,
                         name = attribute.name);
            }
            score += self.brain[i] *
                     attribute.evaluate_values(&original.values[i], &new.values[i]).1
        }
        score
    }

    fn find_all_matches(&self, original: &u32) -> Vec<(f32, u32)> {
        let mut matches: Vec<(f32, u32)> = Vec::new();

        for k in self.products.keys() {
            let score = self.score(original, k);
            matches.push((score, *k));
        }

        matches.sort_by(|&(a, _), &(b, _)| b.partial_cmp(&a).unwrap_or(Ordering::Equal));

        matches
    }
}

fn main() {
    let mut rdr = csv::Reader::from_file("./data/attributes.csv").unwrap().has_headers(false);
    let mut attributes = Vec::new();
    for record in rdr.decode() {
        let (name, values, compare_type, parameter_1, parameter_2): (String, String, u16, u16, u16) =
            record.unwrap();
        let values: Vec<String> = values.split(',').map(|s| s.to_string()).collect();
        let compare_type = match compare_type {
            1 => CompareType::Equal,
            2 => CompareType::GreaterThanOrEqual,
            3 => CompareType::LessThanOrEqual,
            6 => CompareType::IntervalMatch(parameter_1, parameter_2),
            50 => CompareType::LinearInterval(parameter_2, parameter_1),
            _ => panic!("Unknown compare type: {}", compare_type),
        };
        attributes.push(Attribute {
            name: name,
            values: values,
            compare_type: compare_type,
        })
    }

    let mut rdr = csv::Reader::from_file("./data/products.csv").unwrap().has_headers(false);
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

    // let mut wtr = csv::Writer::from_file("./data/records.csv").unwrap();
    // for (id, record) in products.iter() {
    //     let result = wtr.encode((&record.id, &record.name, &record.values));
    //     assert!(result.is_ok());
    // }
    // let result = wtr.flush();
    // assert!(result.is_ok());

    let mut rdr = csv::Reader::from_file("./data/matches.csv").unwrap().has_headers(false);
    let mut matches = Vec::new();
    for record in rdr.decode() {
        let (original, new): (u32, u32) = record.unwrap();

        if products.contains_key(&original) && products.contains_key(&new) {
            matches.push(Match(original, new));
        }
    }

    let seed: &[_] = &[1, 2, 3, 4];
    let mut rng: StdRng = SeedableRng::from_seed(seed);
    rng.shuffle(&mut matches);

    let (train, test) = matches.split_at(((matches.len() as f32) * 0.999) as usize);

    let mut simple_model = SimpleModel {
        attributes: &attributes,
        products: &products,
        matches: &train.to_owned(),
        brain: vec![0.0; attributes.len()],
    };
    simple_model.train();
    println!("{:?}", simple_model.brain);

    simple_model.visualize_score(&33868, &60021);

    let mut positions: Vec<usize> = Vec::new();

    for &Match(original, new) in test.iter() {
        let all_matches = simple_model.find_all_matches(&original);
        // let original_name = &products.get(&original).unwrap().name;
        // let new_name = &products.get(&new).unwrap().name;
        let position = all_matches.iter().position(|&(_, x)| x == new).unwrap();
        positions.push(position);
        let first_few_matches: Vec<String> = all_matches.iter()
            .take(3)
            .map(|&(score, x)| format!("({}, {})", products.get(&x).unwrap().name.clone(), score))
            .collect();
        println!("{}\t({},\t{}):\t{:?}",
                 position,
                 original,
                 new,
                 first_few_matches);
    }

    let sum: usize = positions.iter().sum();
    println!("{}", sum);
}
