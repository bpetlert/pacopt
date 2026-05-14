use alpm::Alpm;
use anyhow::{
    Context,
    Result,
};
use pacmanconf::Config;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Report {
    #[serde(rename = "Name")]
    pub pkg_name: String,

    #[serde(rename = "OptionalDeps")]
    pub deps: Vec<Package>,

    #[serde(skip_serializing)]
    pub installed: bool,

    #[serde(skip_serializing)]
    pub uninstalled: bool,

    #[serde(skip_serializing)]
    pub name_only: bool,

    #[serde(skip_serializing)]
    pub xargs: bool,
}

#[derive(Debug, Serialize)]
pub struct Package {
    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "Description")]
    pub description: String,

    #[serde(rename = "Installed")]
    pub installed: bool,
}

impl Report {
    pub fn new<S: Into<String>>(pkg_name: S) -> Self {
        Self {
            pkg_name: pkg_name.into(),
            installed: false,
            uninstalled: false,
            name_only: false,
            xargs: false,
            deps: Vec::new(),
        }
    }

    pub fn installed(&mut self) {
        self.installed = true;
    }

    pub fn uninstalled(&mut self) {
        self.uninstalled = true;
    }

    pub fn name_only(&mut self) {
        self.name_only = true;
    }

    pub fn xargs(&mut self) {
        self.xargs = true;
    }

    pub fn build(&mut self) -> Result<()> {
        let alpm = {
            let pacman_conf = Config::new().context("Failed to load `pacman.conf`")?;
            Alpm::new(pacman_conf.root_dir, pacman_conf.db_path).context("Could not access ALPM")?
        };

        let pkg = alpm.localdb().pkg(self.pkg_name.as_bytes())?;
        for dep in pkg.optdepends() {
            self.deps.push(Package {
                name: dep.name().into(),
                description: dep.desc().unwrap().into(),
                installed: alpm.localdb().pkg(dep.name()).is_ok(),
            });
        }

        Ok(())
    }
}

impl std::fmt::Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        enum ShowMode {
            All,
            Installed,
            Uninstalled,
        }

        let show_mode = match (self.installed, self.uninstalled) {
            (true, true) => ShowMode::All,
            (true, false) => ShowMode::Installed,
            (false, true) => ShowMode::Uninstalled,
            (false, false) => ShowMode::All,
        };

        let deps: Vec<_> = self
            .deps
            .iter()
            .filter(|p| match show_mode {
                ShowMode::All => true,
                ShowMode::Installed => p.installed,
                ShowMode::Uninstalled => !p.installed,
            })
            .collect();

        if self.xargs {
            write!(
                f,
                "{}",
                deps.iter()
                    .map(|p| p.name.as_str())
                    .collect::<Vec<&str>>()
                    .join(" ")
            )?;
            return Ok(());
        }

        for pkg in deps.iter() {
            if self.name_only {
                writeln!(f, "{name}", name = pkg.name)?;
            } else {
                writeln!(f, "{name}: {desc}", name = pkg.name, desc = pkg.description)?;
            }
        }

        Ok(())
    }
}
