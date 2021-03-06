use crate::{
    core::*,
    domains::Transition,
    fa::QFunction,
    policies::{sample_probs_with_rng, FinitePolicy, Policy},
    utils::argmax_choose,
};
use rand::{rngs::ThreadRng, thread_rng};
use std::f64;

#[inline(always)]
fn kappa(c: f64, x: f64) -> f64 { c / (1.0 + (-x).exp()) }

pub struct TruncatedBoltzmann<Q> {
    q_func: Q,

    c: Parameter,
    rng: ThreadRng,
}

impl<Q> TruncatedBoltzmann<Q> {
    pub fn new<T: Into<Parameter>>(q_func: Q, c: T) -> Self {
        TruncatedBoltzmann {
            q_func,

            c: c.into(),
            rng: thread_rng(),
        }
    }
}

impl<Q> Algorithm for TruncatedBoltzmann<Q> {
    fn handle_terminal(&mut self) { self.c = self.c.step(); }
}

impl<S, Q: QFunction<S>> Policy<S> for TruncatedBoltzmann<Q> {
    type Action = usize;

    fn sample(&mut self, s: &S) -> usize {
        let ps = self.probabilities(s);

        sample_probs_with_rng(&mut self.rng, ps.as_slice().unwrap())
    }

    fn mpa(&mut self, s: &S) -> usize {
        let ps = self.probabilities(s);

        argmax_choose(&mut self.rng, ps.as_slice().unwrap()).1
    }

    fn probability(&mut self, s: &S, a: usize) -> f64 { self.probabilities(s)[a] }
}

impl<S, Q: QFunction<S>> FinitePolicy<S> for TruncatedBoltzmann<Q> {
    fn n_actions(&self) -> usize { self.q_func.n_outputs() }

    fn probabilities(&mut self, s: &S) -> Vector<f64> {
        self.q_func
            .evaluate(&self.q_func.to_features(s))
            .map(|ws| {
                let c = self.c.value();
                let mut z = 0.0;

                let ws: Vector<f64> = ws
                    .into_iter()
                    .map(|v| {
                        let v = kappa(c, *v).exp();
                        z += v;

                        v
                    })
                    .collect();

                ws / z
            })
            .unwrap()
    }
}
