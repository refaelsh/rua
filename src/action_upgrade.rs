use crate::action_install;
use crate::aur_rpc_utils;
use crate::ileft_overs_deleter::ILeftOversDeleter;
use crate::pacman;
use crate::rua_paths::RuaPaths;
use crate::terminal_util;
use alpm::Version;
use colored::*;
use itertools::Itertools;
use lazy_static::lazy_static;
use log::debug;
use log::warn;
use prettytable::format::*;
use prettytable::*;
use regex::Regex;
use std::collections::HashSet;

fn pkg_is_devel(name: &str) -> bool {
	lazy_static! {
		// make sure that the --devel help string in cli_args.rs matches if you change this
		static ref RE: Regex = Regex::new(r"-(git|hg|bzr|svn|cvs|darcs)(-.+)*$").unwrap();
	}
	RE.is_match(name)
}

pub fn upgrade_printonly(devel: bool, ignored: &HashSet<&str>) {
	let alpm = pacman::create_alpm();
	let (outdated, nonexistent) = calculate_upgrade(&alpm, devel, ignored);

	if outdated.is_empty() && nonexistent.is_empty() {
		eprintln!(
			"{}",
			"Good job! All AUR packages are up-to-date.".bright_green()
		);
	} else {
		for (pkg, _, _) in outdated {
			println!("{}", pkg);
		}
		for (pkg, _) in nonexistent {
			println!("{}", pkg);
		}
	}
}

pub fn upgrade_real(
	devel: bool,
	rua_paths: &RuaPaths,
	left_overs_deleter: Box<dyn ILeftOversDeleter>,
	ignored: &HashSet<&str>,
) {
	let alpm = pacman::create_alpm();
	let (outdated, nonexistent) = calculate_upgrade(&alpm, devel, ignored);

	if outdated.is_empty() && nonexistent.is_empty() {
		eprintln!(
			"{}",
			"Good job! All AUR packages are up-to-date.".bright_green()
		);
	} else {
		print_outdated(&outdated, &nonexistent);
		eprintln!();
		loop {
			eprint!("Do you wish to upgrade them? [O]=ok, [X]=exit. ");
			let user_input = terminal_util::read_line_lowercase();
			if &user_input == "o" {
				let outdated: Vec<String> = outdated.iter().map(|o| o.0.to_string()).collect();
				action_install::install(&outdated, rua_paths, left_overs_deleter, false, true);
				break;
			} else if &user_input == "x" {
				break;
			}
		}
	}
}

type OutdatedPkgs<'pkgs> = Vec<(&'pkgs str, String, String)>;
type ForeignPkgs<'pkgs> = Vec<(&'pkgs str, String)>;

fn calculate_upgrade<'pkgs>(
	alpm: &'pkgs alpm::Alpm,
	devel: bool,
	locally_ignored_packages: &HashSet<&str>,
) -> (OutdatedPkgs<'pkgs>, ForeignPkgs<'pkgs>) {
	let pkg_cache = alpm.localdb().pkgs();
	let system_ignored_packages = pacman::get_ignored_packages().unwrap_or_else(|err| {
		warn!("Could not get ignored packages, {}", err);
		HashSet::new()
	});

	let aur_pkgs = pkg_cache
		.iter()
		.filter(|pkg| !pacman::is_installable(&alpm, pkg.name()))
		.map(|pkg| (pkg.name(), pkg.version()))
		.collect::<Vec<_>>();

	let aur_pkgs_string = aur_pkgs
		.iter()
		.map(|&(name, _version)| name)
		.collect::<Vec<_>>()
		.join(" ");
	debug!("You have the following packages outside of main repos installed:");
	debug!("{}", aur_pkgs_string);
	debug!("");

	let mut outdated = Vec::new();
	let mut nonexistent = Vec::new();
	let mut ignored = Vec::new();

	let info_map = aur_rpc_utils::info_map(&aur_pkgs.iter().map(|(p, _)| *p).collect_vec());
	let info_map = info_map.unwrap_or_else(|err| panic!("Failed to get AUR information: {}", err));

	for (pkg, local_ver) in aur_pkgs {
		let raur_ver = info_map.get(pkg).map(|p| p.version.to_string());

		if let Some(raur_ver) = raur_ver {
			if local_ver < Version::new(&*raur_ver) || (devel && pkg_is_devel(pkg)) {
				if locally_ignored_packages.contains(pkg) || system_ignored_packages.contains(pkg) {
					ignored.push(pkg.to_string());
				} else {
					outdated.push((pkg, local_ver.to_string(), raur_ver));
				}
			}
		} else if locally_ignored_packages.contains(pkg) || system_ignored_packages.contains(pkg) {
			ignored.push(pkg.to_string());
		} else {
			nonexistent.push((pkg, local_ver.to_string()));
		}
	}
	if !ignored.is_empty() {
		let ignored_string = ignored.join(" ");
		warn!(
			"The following packages have changed in AUR but are ignored in the system: {}",
			ignored_string
		);
	};

	(outdated, nonexistent)
}

fn print_outdated(outdated: &[(&str, String, String)], nonexistent: &[(&str, String)]) {
	let mut table = Table::new();
	table.set_titles(row![
		"Package".underline(),
		"Current".underline(),
		"Latest".underline()
	]);

	for (pkg, local, remote) in outdated {
		table.add_row(row![pkg.yellow(), local, remote.green(),]);
	}
	for (pkg, local) in nonexistent {
		table.add_row(row![pkg.yellow(), local, "NOT FOUND, ignored".red(),]);
	}
	let fmt: TableFormat = FormatBuilder::new().padding(0, 1).build();
	table.set_format(fmt);
	table.printstd();
}
