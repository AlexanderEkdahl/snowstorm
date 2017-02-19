use std::collections::HashMap;

use models::*;
use std::cmp::Ordering;

pub struct Simple<'a> {
    pub attributes: &'a Vec<Attribute>,
    pub products: &'a HashMap<ProductId, Product>,
    pub matches: &'a Vec<Match>,
    pub brain: Vec<f32>,
}

fn checked_division(dividend: f32, divisor: f32) -> Option<f32> {
    if divisor == 0.0 {
        None
    } else {
        Some(dividend / divisor)
    }
}

impl<'a> Simple<'a> {
    pub fn train(&mut self) {
        let mut temporary_brain: Vec<(ProductId, f32)> = vec![(0, 0.0); self.attributes.len()];

        for &Match(original, new) in self.matches.iter() {
            let original = self.products.get(&original).unwrap();
            let new = self.products.get(&new).unwrap();

            for (i, attribute) in self.attributes.iter().enumerate() {
                if let (true, x) = attribute.evaluate_values(&original.values[i], &new.values[i]) {
                    temporary_brain[i] = (temporary_brain[i].0 + 1, temporary_brain[i].1 + x)
                }
            }
        }

        self.brain = temporary_brain.iter()
            .map(|&(x, y)| checked_division(y, x as f32).unwrap_or(0.0))
            .collect();
    }

    pub fn score(&self, a: &ProductId, b: &ProductId) -> f32 {
        let original = self.products.get(&a).unwrap();
        let new = self.products.get(&b).unwrap();
        let mut score: f32 = 0.0;

        for (i, attribute) in self.attributes.iter().enumerate() {
            score += self.brain[i] *
                     attribute.evaluate_values(&original.values[i], &new.values[i]).1
        }
        score
    }

    pub fn visualize_score(&self, a: &ProductId, b: &ProductId) -> f32 {
        let original = self.products.get(&a).unwrap();
        let new = self.products.get(&b).unwrap();
        let mut score: f32 = 0.0;

        println!("Comparing: {} with {}", &original.name, &new.name);

        for (i, attribute) in self.attributes.iter().enumerate() {
            if original.values[i] != 0 || new.values[i] != 0 {
                println!("{name:>30.*} - {} {:?} {} => {}",
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

    pub fn find_all_matches(&self, original: &ProductId) -> Vec<(f32, ProductId)> {
        let mut matches: Vec<(f32, ProductId)> = Vec::new();

        for k in self.products.keys() {
            let score = self.score(original, k);
            matches.push((score, *k));
        }

        matches.sort_by(|&(a, _), &(b, _)| b.partial_cmp(&a).unwrap_or(Ordering::Equal));

        matches
    }
}