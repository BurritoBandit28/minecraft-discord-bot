use std::env;
use std::ops::{Add, AddAssign, DivAssign, Mul, MulAssign, SubAssign};

pub struct GetSetWrapper<T : Clone + Default> {
    val: T,
}

impl<T : Clone+ Default> GetSetWrapper<T> {
    pub fn set(&mut self, val: T) {
        self.val = val;
    }
    pub fn get(&self) -> T {
        self.val.clone()
    }

    pub fn new() -> Self {
        Self { val: T::default() }
    }
}

impl <T: MulAssign + Clone + Default + AddAssign + DivAssign + SubAssign> GetSetWrapper<T> {
    pub fn add(&mut self, other : T) {
        self.val += other;
    }

    pub fn mul(&mut self, other : T) {
        self.val *= other;
    }

    pub fn div(&mut self, other : T) {
        self.val /= other;
    }

    pub fn sub(&mut self, other : T) {
        self.val -= other;
    }
}

pub fn get_dir_char() -> char {
    if env::consts::OS.eq("windows") {
        return '\\';
    }
    '/'
}

fn get_chain_char() -> char {
    if env::consts::OS.eq("windows") {
        return '&';
    }
    ';'
}