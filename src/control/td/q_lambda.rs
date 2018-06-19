use core::{Algorithm, Controller, Predictor, Shared, Parameter, Vector, Trace};
use domains::Transition;
use fa::{Approximator, MultiLFA, Projection, Projector, QFunction};
use policies::{fixed::Greedy, Policy};
use std::marker::PhantomData;

/// Watkins' Q-learning with eligibility traces.
///
/// # References
/// - Watkins, C. J. C. H. (1989). Learning from Delayed Rewards. Ph.D. thesis,
/// Cambridge University.
/// - Watkins, C. J. C. H., Dayan, P. (1992). Q-learning. Machine Learning,
/// 8:279–292.
pub struct QLambda<S, M: Projector<S>, P: Policy<S>> {
    trace: Trace,

    pub fa_theta: Shared<MultiLFA<S, M>>,

    pub policy: Shared<P>,
    pub target: Greedy<S>,

    pub alpha: Parameter,
    pub gamma: Parameter,

    phantom: PhantomData<S>,
}

impl<S: 'static, M: Projector<S> + 'static, P: Policy<S>> QLambda<S, M, P> {
    pub fn new<T1, T2>(
        trace: Trace,
        fa_theta: Shared<MultiLFA<S, M>>,
        policy: Shared<P>,
        alpha: T1,
        gamma: T2,
    ) -> Self
    where
        T1: Into<Parameter>,
        T2: Into<Parameter>,
    {
        QLambda {
            trace: trace,

            fa_theta: fa_theta.clone(),

            policy: policy,
            target: Greedy::new(fa_theta),

            alpha: alpha.into(),
            gamma: gamma.into(),

            phantom: PhantomData,
        }
    }
}

impl<S, M: Projector<S>, P: Policy<S, Action = usize>> Algorithm<S, usize> for QLambda<S, M, P> {
    fn handle_sample(&mut self, t: &Transition<S, usize>) {
        let (s, ns) = (t.from.state(), t.to.state());

        let phi_s = self.fa_theta.borrow().projector.project(s);

        let qs = self.fa_theta.borrow().evaluate_phi(&phi_s);
        let nqs = self.fa_theta.borrow().evaluate(ns).unwrap();

        let td_error = t.reward + self.gamma * nqs[self.target.sample(&ns)] - qs[t.action];

        if t.action == self.target.sample(&s) {
            let rate = self.trace.lambda.value() * self.gamma.value();
            self.trace.decay(rate);
        } else {
            self.trace.decay(0.0);
        }

        self.trace
            .update(&phi_s.expanded(self.fa_theta.borrow().projector.dim()));
        self.fa_theta.borrow_mut().update_action_phi(
            &Projection::Dense(self.trace.get()),
            t.action,
            td_error * self.alpha,
        );
    }

    fn handle_terminal(&mut self, t: &Transition<S, usize>) {
        self.alpha = self.alpha.step();
        self.gamma = self.gamma.step();

        self.trace.decay(0.0);

        self.target.handle_terminal(t);
        self.policy.borrow_mut().handle_terminal(t);
    }
}

impl<S, M: Projector<S>, P: Policy<S, Action = usize>> Controller<S, usize> for QLambda<S, M, P> {
    fn pi(&mut self, s: &S) -> usize { self.target.sample(s) }
    fn mu(&mut self, s: &S) -> usize { self.policy.borrow_mut().sample(s) }
}

impl<S, M: Projector<S>, P: Policy<S, Action = usize>> Predictor<S, usize> for QLambda<S, M, P> {
    fn v(&mut self, s: &S) -> f64 {
        let a = self.pi(s);

        self.qsa(s, a)
    }

    fn qs(&mut self, s: &S) -> Vector<f64> {
        self.fa_theta.borrow().evaluate(s).unwrap()
    }

    fn qsa(&mut self, s: &S, a: usize) -> f64 {
        self.fa_theta.borrow().evaluate_action(&s, a)
    }
}
