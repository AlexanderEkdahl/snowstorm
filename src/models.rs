#[derive(Debug)]
pub struct Attribute {
    pub name: String,
    pub values: Vec<String>,
    pub compare_type: CompareType,
}

impl Attribute {
    pub fn map_value(&self, value: Option<String>) -> u16 {
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

    pub fn evaluate_values(&self, a: &u16, b: &u16) -> (bool, f32) {
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
                 if *b <= *a + up && *b >= if down >= *a { 0 } else { *a - down } {
                     if *b > *a && up > 0 {
                         1.0 - (*b - *a) as f32 * 0.5 / up as f32
                     } else if *a > *b && down > 0 {
                         1.0 - (*a - *b) as f32 * 0.5 / down as f32
                     } else {
                         1.0
                     }
                 } else {
                     0.0
                 }
             }
         })
    }
}

#[derive(Debug)]
pub enum CompareType {
    Equal,
    GreaterThanOrEqual,
    LessThanOrEqual,
    IntervalMatch(u16, u16),
    LinearInterval(u16, u16),
}

pub type ProductId = u32;

#[derive(Debug)]
pub struct Product {
    pub id: ProductId,
    pub name: String,
    pub values: Vec<u16>,
}

#[derive(Clone, Debug)]
pub struct Match(pub ProductId, pub ProductId);