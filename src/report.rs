use alpm::{
    Alpm,
    Dep,
    SigLevel,
};
use anyhow::{
    Context,
    Result,
    anyhow,
};
use pacmanconf::Config;
use serde::Serialize;
use tabled::{
    Table,
    Tabled,
    settings::{
        Alignment,
        Modify,
        Style,
        location::ByColumnName,
    },
};

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

    #[serde(skip_serializing)]
    alpm: Alpm,
}

#[derive(Clone, Debug, Serialize, Tabled)]
pub struct Package {
    #[serde(rename = "Name")]
    #[tabled(order = 1, rename = "Name")]
    pub name: String,

    #[serde(rename = "Provider")]
    #[tabled(skip)]
    pub provider: String,

    #[serde(rename = "Description")]
    #[tabled(order = 2, rename = "Description")]
    pub description: String,

    #[serde(rename = "Installed")]
    #[tabled(order = 0, rename = "Installed", display = "display_installed")]
    pub installed: bool,
}

fn display_installed(installed: &bool) -> String {
    if *installed {
        "✔️".into()
    } else {
        "❌".into()
    }
}

impl Report {
    pub fn new<S: Into<String>>(pkg_name: S) -> Result<Self> {
        let alpm = {
            let pacman_conf = Config::new().context("Failed to load `pacman.conf`")?;
            let alpm = Alpm::new(pacman_conf.root_dir, pacman_conf.db_path)
                .context("Could not access ALPM")?;

            // Register repository database
            for repo in &pacman_conf.repos {
                alpm.register_syncdb(&*repo.name, SigLevel::USE_DEFAULT)
                    .with_context(|| format!("Could not register `{}`", repo.name))?;
            }

            alpm
        };

        Ok(Self {
            pkg_name: pkg_name.into(),
            installed: false,
            uninstalled: false,
            name_only: false,
            xargs: false,
            deps: Vec::new(),
            alpm,
        })
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

    pub fn provider(&self, dep: &Dep) -> Result<Package> {
        if let Ok(pkg) = self.alpm.localdb().pkg(dep.name()) {
            return Ok(Package {
                name: dep.name().into(),
                provider: dep.name().into(),
                description: dep
                    .desc()
                    .unwrap_or_else(|| pkg.desc().unwrap_or_default())
                    .into(),
                installed: true,
            });
        }

        // Search in localdb (installed)
        for pkg in self.alpm.localdb().pkgs().iter() {
            if !pkg.provides().is_empty() && pkg.provides().iter().any(|d| d.name() == dep.name()) {
                return Ok(Package {
                    name: dep.name().into(),
                    provider: pkg.name().into(),
                    description: dep
                        .desc()
                        .unwrap_or_else(|| pkg.desc().unwrap_or_default())
                        .into(),
                    installed: true,
                });
            }
        }

        // Search in all syncdbs
        for db in self.alpm.syncdbs().iter() {
            if let Ok(pkg) = db.pkg(dep.name()) {
                return Ok(Package {
                    name: dep.name().into(),
                    provider: pkg.name().into(),
                    description: dep
                        .desc()
                        .unwrap_or_else(|| pkg.desc().unwrap_or_default())
                        .into(),
                    installed: false,
                });
            }
        }

        Ok(Package {
            name: dep.name().into(),
            provider: String::new(),
            description: dep.desc().unwrap_or_default().into(),
            installed: false,
        })
    }

    pub fn build(&mut self) -> Result<()> {
        let Ok(pkg) = self.alpm.localdb().pkg(self.pkg_name.as_bytes()) else {
            return Err(anyhow!("Package `{}` is not installed.", self.pkg_name));
        };

        for dep in pkg.optdepends() {
            let provider = self.provider(dep)?;
            self.deps.push(Package {
                name: provider.name,
                provider: provider.provider,
                description: provider.description,
                installed: provider.installed,
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

        let deps = self
            .deps
            .iter()
            .filter(|p| match show_mode {
                ShowMode::All => true,
                ShowMode::Installed => p.installed,
                ShowMode::Uninstalled => !p.installed,
            })
            .cloned()
            .map(|p| {
                let name = if p.name != p.provider && !p.provider.is_empty() {
                    format!("{} ({})", p.name, p.provider)
                } else {
                    p.name
                };

                Package {
                    name,
                    provider: p.provider,
                    description: p.description,
                    installed: p.installed,
                }
            })
            .collect::<Vec<_>>();

        if deps.is_empty() {
            return Ok(());
        }

        if self.xargs {
            write!(
                f,
                "{}",
                deps.iter()
                    .map(|p| p.provider.as_str())
                    .collect::<Vec<&str>>()
                    .join(" ")
            )?;
            return Ok(());
        }

        if self.name_only {
            for pkg in deps.iter() {
                writeln!(f, "{name}", name = pkg.provider)?;
            }
            return Ok(());
        }

        let mut table = Table::new(deps);
        table
            .with(Style::re_structured_text())
            .with(Modify::new(ByColumnName::new("Name")).with(Alignment::left()))
            .with(Modify::new(ByColumnName::new("Description")).with(Alignment::left()))
            .with(Modify::new(ByColumnName::new("Installed")).with(Alignment::center()));
        writeln!(f, "{table}")?;

        Ok(())
    }
}
