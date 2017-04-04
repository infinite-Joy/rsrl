use std::f64;
use rand::{Rng, thread_rng, ThreadRng};
use super::Policy;


pub struct Boltzmann {
    tau: f64,
    rng: ThreadRng,
}

impl Boltzmann {
    pub fn new<T: Into<f64>>(tau: T) -> Self {
        Boltzmann {
            tau: tau.into(),
            rng: thread_rng(),
        }
    }

    pub fn probabilities(&self, qs: &[f64]) -> Vec<f64> {
        let mq = qs.iter().cloned().fold(f64::MIN, f64::max);
        let ws: Vec<f64> = qs.iter()
            .cloned()
            .map(|q| ((q - mq) / self.tau).exp())
            .collect();

        let z: f64 = ws.iter().sum();
        ws.iter().map(|w| w / z).collect()
    }
}

impl Policy for Boltzmann {
    fn sample(&mut self, s: &[f64]) -> usize {
        let ps = self.probabilities(s);

        let r = self.rng.next_f64();
        match ps.iter().position(|p| *p > r) {
            Some(index) => index,
            None => ps.len() - 1,
        }
    }
}