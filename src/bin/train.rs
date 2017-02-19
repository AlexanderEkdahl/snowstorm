extern crate snowstorm;
extern crate csv;
extern crate rand;

use rand::{Rng, SeedableRng, StdRng};

use self::snowstorm::models::Match;
use self::snowstorm::simple::*;
use self::snowstorm::parse;

fn main() {
    let attributes = parse::attributes().unwrap();
    let products = parse::products(&attributes).unwrap();
    let mut matches = parse::matches(&products).unwrap();

    // let mut wtr = csv::Writer::from_file("./data/records.csv").unwrap();
    // for (id, record) in products.iter() {
    //     let result = wtr.encode((&record.id, &record.name, &record.values));
    //     assert!(result.is_ok());
    // }
    // let result = wtr.flush();
    // assert!(result.is_ok());

    let seed: &[_] = &[1, 2, 3, 4];
    let mut rng: StdRng = SeedableRng::from_seed(seed);
    rng.shuffle(&mut matches);

    let (train, test) = matches.split_at(((matches.len() as f32) * 0.9999) as usize);

    let mut simple_model = Simple {
        attributes: &attributes,
        products: &products,
        matches: &train.to_owned(),
        brain: vec![0.0; attributes.len()],
    };
    simple_model.train();

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
