//! Agent policy module.
//!
//! This module contains [fixed](fixed/index.html) and
//! [parameterised](parameterised/index.html) policies for reinforcement
//! learning control problems. A policy is considered to be either deterministic
//! or stochastic, for which we have the following definitions, respectively: 1)
//! _π(X) -> U_; 2) _π(X, U) -> R_, or equivalently, _π(x) -> Ω(U)_;
//!
//! where _X_ and _U_ are the state and action spaces, respectively; _R_ is the
//! set of real values; and _Ω(U)_ is the set of probability measures on the
//! action set _U_. In general, deterministic policies may be considered a
//! special case of stochastic policies in which all probability mass
//! is placed on a single action _u'_ for any given state _x_. For continuous
//! policies, this can be seen as a dirac delta distribution, _δ(u' - u)_.
use crate::{core::*, domains::Transition, fa::Parameterised};
use rand::{seq::SliceRandom, thread_rng, Rng};

#[allow(dead_code)]
#[inline]
pub(self) fn sample_probs(probabilities: &[f64]) -> usize {
    sample_probs_with_rng(&mut thread_rng(), probabilities)
}

#[inline]
pub(self) fn sample_probs_with_rng(rng: &mut impl Rng, probabilities: &[f64]) -> usize {
    let r = rng.gen::<f64>();

    match probabilities
        .into_iter()
        .scan(0.0, |state, &p| {
            *state = *state + p;

            Some(*state)
        })
        .position(|p| p > r)
    {
        Some(index) => index,
        None => probabilities.len() - 1,
    }
}

/// Policy trait for functions that define a probability distribution over
/// actions.
pub trait Policy<S>: Algorithm {
    type Action;

    /// Sample the (possibly stochastic) policy distribution for a given input.
    fn sample(&mut self, input: &S) -> Self::Action { self.mpa(input) }

    /// Return the "most probable action" according to the policy distribution,
    /// if well-defined.
    fn mpa(&mut self, _: &S) -> Self::Action { unimplemented!() }

    /// Return the probability of selecting an action for a given input.
    fn probability(&mut self, input: &S, a: Self::Action) -> f64;
}

/// Trait for policies that are defined on a finite action space.
pub trait FinitePolicy<S>: Policy<S, Action = usize> {
    fn n_actions(&self) -> usize;

    /// Return the probability of selecting each action for a given input.
    fn probabilities(&mut self, input: &S) -> Vector<f64>;
}

/// Trait for policies that have a differentiable representation.
pub trait DifferentiablePolicy<S>: Policy<S> {
    /// Compute the derivative of the log probability for a single action.
    fn grad_log(&self, input: &S, a: Self::Action) -> Matrix<f64>;
}

/// Trait for policies that are parameterised by a vector of weights.
pub trait ParameterisedPolicy<S>: Policy<S> + Parameterised {
    /// Update the weights in the direction of an error for a given state and
    /// action.
    fn update(&mut self, input: &S, a: Self::Action, error: f64);

    /// Update the weights directly using an update matrix.
    fn update_raw(&mut self, errors: Matrix<f64>);
}

pub mod fixed;
pub mod parameterised;

import_all!(perturbation);

// Shared<T> impls:
impl<S, T: Policy<S>> Policy<S> for Shared<T> {
    type Action = T::Action;

    fn sample(&mut self, input: &S) -> Self::Action {
        self.borrow_mut().sample(input)
    }

    fn mpa(&mut self, s: &S) -> Self::Action {
        self.borrow_mut().mpa(s)
    }

    fn probability(&mut self, input: &S, a: Self::Action) -> f64 {
        self.borrow_mut().probability(input, a)
    }
}

impl<S, T: FinitePolicy<S>> FinitePolicy<S> for Shared<T> {
    fn n_actions(&self) -> usize {
        self.borrow().n_actions()
    }

    fn probabilities(&mut self, input: &S) -> Vector<f64> {
        self.borrow_mut().probabilities(input)
    }
}

impl<S, T: DifferentiablePolicy<S>> DifferentiablePolicy<S> for Shared<T> {
    fn grad_log(&self, input: &S, a: Self::Action) -> Matrix<f64> {
        self.borrow().grad_log(input, a)
    }
}

impl<S, T: ParameterisedPolicy<S>> ParameterisedPolicy<S> for Shared<T> {
    fn update(&mut self, input: &S, a: Self::Action, error: f64) {
        self.borrow_mut().update(input, a, error)
    }

    fn update_raw(&mut self, errors: Matrix<f64>) {
        self.borrow_mut().update_raw(errors)
    }
}
