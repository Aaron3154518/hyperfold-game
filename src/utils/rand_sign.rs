use rand::{rngs::ThreadRng, Rng};

pub trait RandSign {
    fn gen_sign(&mut self) -> i8;
}

impl RandSign for ThreadRng {
    fn gen_sign(&mut self) -> i8 {
        match self.gen_bool(0.5) {
            true => 1,
            false => -1,
        }
    }
}
