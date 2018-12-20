use core::*;
use domains::Transition;
use fa::{Approximator, VFunction, Parameterised, Projector, Projection, SimpleLFA};
use geometry::Space;
use ndarray::Axis;
use ndarray_linalg::solve::Solve;
use utils::{argmaxima, pinv};

pub struct LSTDLambda<S, P: Projector<S>> {
    pub fa_theta: Shared<SimpleLFA<S, P>>,

    pub gamma: Parameter,

    trace: Trace,

    a: Matrix<f64>,
    b: Vector<f64>,
}

impl<S, P: Projector<S>> LSTDLambda<S, P> {
    pub fn new<T: Into<Parameter>>(fa_theta: Shared<SimpleLFA<S, P>>,
                                   trace: Trace,
                                   gamma: T) -> Self {
        let n_features = fa_theta.borrow().projector.dim();

        LSTDLambda {
            fa_theta,

            gamma: gamma.into(),

            trace,

            a: Matrix::zeros((n_features, n_features)),
            b: Vector::zeros((n_features,)),
        }
    }
}

impl<S, P: Projector<S>> LSTDLambda<S, P> {
    #[inline(always)]
    fn compute_dense_fv(&self, s: &S) -> Vector<f64> {
        self.fa_theta.borrow().projector.project(s).expanded(self.a.rows())
    }

    #[inline]
    fn update_trace(&mut self, phi: &Vector<f64>) -> Vector<f64> {
        let decay_rate = self.trace.lambda.value() * self.gamma.value();

        self.trace.decay(decay_rate);
        self.trace.update(phi);

        self.trace.get()
    }

    pub fn solve(&mut self) {
        // First try the clean approach:
        if let Ok(theta) = self.a.solve(&self.b) {
            self.fa_theta.borrow_mut().approximator.weights.assign(&theta);

        // Otherwise solve via SVD:
        } else if let Ok(ainv) = pinv(&self.a) {
            let theta = ainv.dot(&self.b);

            self.fa_theta.borrow_mut().approximator.weights.assign(&theta);
        }
    }
}

impl<S, P: Projector<S>> Algorithm for LSTDLambda<S, P> {
    fn handle_terminal(&mut self) {
        self.gamma = self.gamma.step();
    }
}

impl<S, A, P: Projector<S>> BatchLearner<S, A> for LSTDLambda<S, P> {
    fn handle_batch(&mut self, ts: &[Transition<S, A>]) {
        ts.into_iter().for_each(|t| {
            let (s, ns) = (t.from.state(), t.to.state());

            let phi_s = self.compute_dense_fv(s);
            let z = self.update_trace(&phi_s);

            self.b.scaled_add(t.reward, &z);

            let pd = if t.terminated() {
                self.trace.decay(0.0);

                phi_s
            } else {
                let phi_ns = self.compute_dense_fv(ns);

                phi_s - self.gamma.value()*phi_ns
            }.insert_axis(Axis(0));

            self.a += &z.insert_axis(Axis(1)).dot(&pd);
        });

        self.solve();
    }
}

impl<S, P: Projector<S>> ValuePredictor<S> for LSTDLambda<S, P> {
    fn predict_v(&mut self, s: &S) -> f64 {
        self.fa_theta.borrow().evaluate(s).unwrap()
    }
}

impl<S, A, P: Projector<S>> ActionValuePredictor<S, A> for LSTDLambda<S, P> {}

impl<S, P: Projector<S>> Parameterised for LSTDLambda<S, P> {
    fn weights(&self) -> Matrix<f64> {
        self.fa_theta.borrow().weights()
    }
}
