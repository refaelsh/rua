use crate::folder_deleter::IFolderDeleter;
use crate::ileft_overs_deleter::ILeftOversDeleter;
use crate::rua_paths::RuaPaths;
use std::path::PathBuf;

pub struct LeftOversDeleter {
	m_folder_deleter: Box<dyn IFolderDeleter>,
}

impl LeftOversDeleter {
	pub fn new(folder_deleter: Box<dyn IFolderDeleter>) -> LeftOversDeleter {
		LeftOversDeleter {
			m_folder_deleter: folder_deleter,
		}
	}
}

impl ILeftOversDeleter for LeftOversDeleter {
	fn delete_folders(&self, targets: &[String], rua_paths: &RuaPaths) -> rm_rf::Result<()> {
		let build_folder = &rua_paths
			.global_build_dir
			.as_path()
			.to_str()
			.unwrap()
			.to_owned();

		for target in targets {
			let path_for_deletion = build_folder.clone() + "/" + target;
			let res = self
				.m_folder_deleter
				.delete_folder(&PathBuf::from(&path_for_deletion));
			match res {
				Ok(()) => (),
				Err(error) => return Err(error),
			}
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use crate::folder_deleter::*;
	use crate::ileft_overs_deleter::ILeftOversDeleter;
	use crate::left_overs_deleter::LeftOversDeleter;
	use crate::rua_paths::RuaPaths;
	use std::path::PathBuf;

	#[test]
	fn delete_folders_test() {
		let mut mock_folder_deleter = Box::new(MockIFolderDeleter::new());

		mock_folder_deleter
			.expect_delete_folder()
			.withf(|x: &PathBuf| {
				assert_eq!(
					x.as_path().to_str().unwrap(),
					"/home/refaelsh/.cache/rua/build/some_target"
				);
				true
			})
			.returning(|_| Ok(()));

		let left_overs_deleter = LeftOversDeleter::new(mock_folder_deleter);

		let targets = ["some_target".to_string()];
		let rua_paths = RuaPaths::initialize_paths();

		let res = left_overs_deleter.delete_folders(&targets, &rua_paths);
		match res {
			Ok(()) => (),
			Err(_error) => panic!(),
		}
	}
}
