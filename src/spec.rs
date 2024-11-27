use cpython::{ObjectProtocol, PyObject, Python};

#[derive(Debug)]
pub struct Spec {
	pub reward_threshold: f64,
	pub max_episode_steps: i32,
}

impl Spec {
	pub fn extract_data(pyo: &PyObject) -> Spec {
		let gil = Python::acquire_gil();
		let py = gil.python();
		Spec {
			reward_threshold: pyo
				.getattr(py, "reward_threshold")
				.expect("Unable to extract reward_threshold")
				.extract(py)
				.expect("Unable to extract reward_threshold"),
			max_episode_steps: pyo
				.getattr(py, "max_episode_steps")
				.expect("Unable to extract max_episode_steps")
				.extract(py)
				.expect("Unable to extract max_episode_steps"),
		}
	}
}
