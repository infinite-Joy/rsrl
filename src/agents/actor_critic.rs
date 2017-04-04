use super::Agent;

use {Function, Parameterised};
use domain::Transition;
use geometry::{Space, ActionSpace};
use policies::{Policy, Greedy};


pub struct ActorCritic<Q, V, P> {
    actor: Q,
    critic: V,

    policy: P,

    alpha: f64,
    beta: f64,
    gamma: f64,
}

impl<Q, V, P> ActorCritic<Q, V, P> {
    pub fn new(actor: Q, critic: V, policy: P, alpha: f64, beta: f64, gamma: f64) -> Self {
        ActorCritic {
            actor: actor,
            critic: critic,

            policy: policy,

            alpha: alpha,
            beta: beta,
            gamma: gamma,
        }
    }
}

impl<S: Space, Q, V, P: Policy> Agent<S> for ActorCritic<Q, V, P>
    where V: Function<S::Repr, f64> + Parameterised<S::Repr, f64>,
          Q: Function<S::Repr, Vec<f64>> + Parameterised<S::Repr, [f64]>
{
    fn handle(&mut self, t: &Transition<S, ActionSpace>) -> usize {
        let (s, ns) = (t.from.get(), t.to.get());

        let delta = t.reward +
            self.gamma*self.critic.evaluate(ns) - self.critic.evaluate(s);

        let mut errors = vec![0.0; self.actor.n_outputs()];
        errors[t.action] = self.beta*delta;

        self.actor.update(s, errors.as_slice());
        self.critic.update(s, &(self.alpha*delta));

        self.policy.sample(self.actor.evaluate(ns).as_slice())
    }
}