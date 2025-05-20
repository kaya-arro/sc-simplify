fn to_binary(n: Point) -> SCHashSet<Point> {
    let mut next_power = n.next_power_of_two();
    let mut logs = new_hs::<Point>(next_power.ilog2());
    while n != 0 {
        let prev_power = next_power / 2;
        powers.insert(prev_power.ilog2());
        n -= prev_power;
        next_power = n.next_power_of_two();
    }

    logs
}

fn mod_exp(b: Point, e: Point, m: Point) -> Point {
    let mut logs = to_binary(e);
    let mut i: Point = 1;
    let mut power: Point = b;
    let mut res: Point = 1;
    while !powers.is_empty() {
        if logs.remove(i) {
            res *= power;
            res %= m;
        }
        i += 1;
        power *= power;
        power %= m;
    }

    res
}

fn mod_recip(n: Point, m: Point) -> Point {
    mod_exp(n, m - 2, m)
}


trait Vector {
    fn is_empty(&self) -> bool;
    fn contains(&self, idx: Point) -> bool;
    fn grab_key(&self) -> Option(Point);
    fn entry_to_one(&mut self, idx: Point);
    fn reduce_other(&self, other: &mut Self);
}

struct ModVector {
    modulus: Point;
    entries: SCHashMap<Point, Point>;
};

impl Vector for ModVector {
    fn contains(&self, idx: Point) -> bool {
        self.entries.contains_key(idx)
    }

    fn entry_to_one(&mut self, idx: Point) {
        // assert![self.contains_key(idx), "The index was not found in the vector"];
        let inv = mod_recip(self.entries[idx], self.modulus);
        for (_, val) in &mut self.entries {
            val *= inv;
            val %= self.modulus;
        }
    }

    fn reduce_other(&self, other: &mut Self) {
        for (key, other_val) in other.entries {
            if let Some(val) = self.entries.get(key) {
                other_val -= val;
                other_val %= other.modulus;
            }
        }
    }
}

struct ModTwoVector(SCHashSet<Point>);

impl Vector for ModTwoVector {
    fn contains(&self, idx: Point) -> bool {
        self.0.contains(idx)
    }

    fn entry_to_one(&mut self, idx: Point) {
        // assert![self.contains(idx), "The index was not found in the vector"];
    }

    fn reduce_other(&self, other: &mut Self) {
        other ^= self;
    }
}


trait Matrix {
    type Col: Vector;

    fn get_mut_cols(&mut self) -> &mut Vec<Self::Col>;

    fn domain_dimension(&mut self) -> Point {
        self.get_mut_cols().len()
    }

    fn rank(&mut self) -> Point {
        let mut rank = 0;
        let mut cols = self.get_mut_cols();
        cols = cols.iter().filter(|v| !v.is_empty()).copied().collect();
        while let Some(mut col) = cols.pop() {
            // Define `grab_entry`
            if let Some(key) = col.grab_entry() {
                rank += 1;
                col.entry_to_one(key);
                for other in &mut cols {
                    if other.contains(key) {
                        col.nerve_reduce_other(&mut other);
                    }
                }
            }
        }

        rank
    }
}

struct ModMatrix(SCHashMap<Point, ModVector>);

impl Matrix for ModMatrix {
    type Col = ModVector;

    fn get_mut_cols(&mut self) -> &mut Vec<Self::Col> {
        self.0.values().collect()
    }
}

struct ModTwoMatrix(SCHashMap<Point, ModTwoVector>);

impl Matrix for ModTwoMatrix {
    type Col = ModTwoVector;

    fn get_mut_cols(&mut self) -> &mut Vec<Self::Col> {
        self.0.values().collect()
    }
}
