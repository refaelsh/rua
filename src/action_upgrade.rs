use crate::pacman;
use crate::{action_install, terminal_util};
use directories::ProjectDirs;

pub fn upgrade(dirs: &ProjectDirs) {
	let alpm = pacman::create_alpm();
	let local_db = alpm.local_db();
	let pkg_cache = local_db.pkg_cache();
	let aur_pkgs = pkg_cache
		.iter()
		.filter(|pkg| !pacman::is_package_installable(&alpm, pkg.name()))
		.map(|pkg| (pkg.name(), pkg.version().to_string()))
		.collect::<Vec<_>>();
	let aur_pkgs_string = aur_pkgs
		.iter()
		.map(|(pkg, _ver)| *pkg)
		.collect::<Vec<_>>()
		.join(" ");
	eprintln!("You have the following packages outside of main repos installed:");
	eprintln!("{}", aur_pkgs_string);
	eprintln!();
	let mut up_to_date = Vec::new();
	let mut outdated = Vec::new();
	let mut unexistent = Vec::new();
	for (pkg, local_ver) in aur_pkgs {
		let raur_ver = action_install::raur_info(pkg).map(|p| p.version);
		if let Some(raur_ver) = raur_ver {
			if local_ver == raur_ver {
				up_to_date.push(pkg);
			} else {
				outdated.push((pkg, local_ver, raur_ver));
			}
		} else {
			unexistent.push(pkg);
		}
	}
	if !unexistent.is_empty() {
		eprintln!("The following packages do not seem to exist in neither AUR nor main repos:");
		eprintln!("{}", unexistent.join(" "));
		eprintln!("Consider deleting them or verifying if they are really in use.");
		eprintln!();
	}
	if outdated.is_empty() {
		eprintln!("All AUR packages are up-to-date. Congratulations!");
	} else {
		eprintln!("The following AUR packages have upstream upgrades:");
		for (pkg, local, remote) in &outdated {
			eprintln!("{}  {} -> {}", pkg, local, remote);
		}
		eprintln!();
		let outdated: Vec<String> = outdated.iter().map(|o| o.0.to_string()).collect();
		loop {
			eprint!("Do you wish to upgrade them? [O]=ok, [X]=exit. ");
			let string = terminal_util::read_line_lowercase();
			if string == "o" {
				action_install::install(&outdated, dirs, false, true);
			} else if string == "x" {
				break;
			}
		}
	}
}