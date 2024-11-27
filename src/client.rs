use cpython::{GILGuard, ObjectProtocol, PyDict, PyModule, Python};

use crate::space_template::SpaceTemplate;
use crate::{environment::Environment, error::GymError};

pub struct GymClient {
	pub gil: GILGuard,
	pub gym: PyModule,
	pub version: String,
}

pub enum RenderMode {
	Human,
	RgbArray,
	Custom(String),
}

use std::fmt;

impl fmt::Display for RenderMode {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let s = match self {
			RenderMode::Human => "human",
			RenderMode::RgbArray => "rgb_array",
			RenderMode::Custom(s) => s,
		};
		write!(f, "{}", s)
	}
}

#[derive(Default)]
pub struct MakeOptions {
	pub render_mode: Option<RenderMode>,
	// pub apply_api_compatibility: bool,
	// pub use_old_gym_enviroment: bool,
}

impl Default for GymClient {
	fn default() -> Self {
		GymClient::new(None)
	}
}

impl GymClient {
	pub fn new(venv: Option<&str>) -> Self {
		// Get python
		let gil = Python::acquire_gil();
		let py = gil.python();

		// venv
		let sys = py.import("sys").expect("Error: import sys");

		if let Some(venv) = venv {
			let path = sys.get(py, "path").expect("Error: sys.path");
			path.call_method(py, "append", (venv,), None)
				.unwrap_or_else(|_| panic!("Error: sys.path.append{venv}"));
		}

		let gym = py.import("gymnasium").expect("Error: import gymnasium");

		let version = gym
			.get(py, "__version__")
			.expect("Unable to call gym.__version__")
			.extract(py)
			.expect("Unable to call gym.__version__");
		Self { gil, gym, version }
	}

	pub fn make(&self, mut env_id: &str, options: Option<MakeOptions>) -> Result<Environment, GymError> {
		let py = self.gil.python();
		let dict = PyDict::new(py);
		if let Some(options) = options {
			// dict.set_item(py, "apply_api_compatibility", options.apply_api_compatibility)
			// 	.expect("Unable to set apply_api_compatibility");
			if let Some(render_mode) = options.render_mode {
				dict.set_item(py, "render_mode", render_mode.to_string())
					.map_err(|_| GymError::InvalidRenderMode)?;
			}
			// if options.use_old_gym_enviroment {
			// 	dict.set_item(py, "env_id", env_id).expect("Unable to set env_id");
			// 	env_id = "GymV26Environment-v0";
			// }
		}
		let env = self
			.gym
			.call(py, "make", (env_id,), Some(&dict))
			.map_err(|e| GymError::InvalidMake(env_id.to_owned(), dict.items(py), e))?;

		Ok(Environment {
			gil: &self.gil,
			observation_space: SpaceTemplate::extract_template(
				&env.getattr(py, "observation_space")
					.expect("Unable to get attribute 'observation_space'"),
			),
			action_space: SpaceTemplate::extract_template(
				&env.getattr(py, "action_space")
					.expect("Unable to get attribute 'action_space'"),
			),
			env,
		})
	}

	pub fn list_all(&self) -> Vec<String> {
		let py = self.gil.python();
		// gymnasiun.envs.registry.keys()
		self.gym
			.get(py, "envs")
			.expect("Unable to call gym.envs")
			.getattr(py, "registry")
			.expect("Unable to get attribute 'all'")
			.cast_as::<PyDict>(py)
			.unwrap()
			.items(py)
			.iter()
			.map(|(k, _)| k.extract::<String>(py).unwrap())
			.collect()
	}

	pub fn version(&self) -> &str {
		self.version.as_str()
	}
}
