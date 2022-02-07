// .moon/workspace.yml

mod node;
mod vcs;

use crate::constants;
use crate::errors::map_figment_error_to_validation_errors;
use crate::types::FilePath;
use crate::validators::{validate_child_relative_path, validate_id};
use figment::value::{Dict, Map};
use figment::{
    providers::{Format, Yaml},
    Figment, Metadata, Profile, Provider,
};
pub use node::{NodeConfig, NpmConfig, PackageManager, PnpmConfig, YarnConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use validator::{Validate, ValidationError, ValidationErrors};
pub use vcs::{VcsConfig, VcsManager};

// Validate the `projects` field is a map of valid file system paths
// that are relative from the workspace root. Will fail on absolute
// paths ("/"), and parent relative paths ("../").
fn validate_projects(projects: &HashMap<String, FilePath>) -> Result<(), ValidationError> {
    for (key, value) in projects {
        validate_id(&format!("projects.{}", key), key)?;

        match validate_child_relative_path("projects", value) {
            Ok(_) => {}
            Err(e) => return Err(e),
        }
    }

    Ok(())
}

#[derive(Debug, Deserialize, PartialEq, Serialize, Validate)]
pub struct WorkspaceConfig {
    #[validate]
    pub node: Option<NodeConfig>,

    #[validate(custom = "validate_projects")]
    pub projects: HashMap<String, FilePath>,

    #[validate]
    pub vcs: Option<VcsConfig>,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        WorkspaceConfig {
            node: Some(NodeConfig::default()),
            projects: HashMap::new(),
            vcs: Some(VcsConfig::default()),
        }
    }
}

impl Provider for WorkspaceConfig {
    fn metadata(&self) -> Metadata {
        Metadata::named(constants::CONFIG_WORKSPACE_FILENAME)
    }

    fn data(&self) -> Result<Map<Profile, Dict>, figment::Error> {
        figment::providers::Serialized::defaults(WorkspaceConfig::default()).data()
    }

    fn profile(&self) -> Option<Profile> {
        Some(Profile::Default)
    }
}

impl WorkspaceConfig {
    pub fn load(path: PathBuf) -> Result<WorkspaceConfig, ValidationErrors> {
        let config: WorkspaceConfig = match Figment::from(WorkspaceConfig::default())
            .merge(Yaml::file(path))
            .extract()
        {
            Ok(cfg) => cfg,
            Err(error) => return Err(map_figment_error_to_validation_errors(&error)),
        };

        if let Err(errors) = config.validate() {
            return Err(errors);
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::tests::handled_jailed_error;

    fn load_jailed_config() -> Result<WorkspaceConfig, figment::Error> {
        match WorkspaceConfig::load(PathBuf::from(constants::CONFIG_WORKSPACE_FILENAME)) {
            Ok(cfg) => Ok(cfg),
            Err(errors) => Err(handled_jailed_error(&errors)),
        }
    }

    #[test]
    fn loads_defaults() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(constants::CONFIG_WORKSPACE_FILENAME, "projects: {}")?;

            let config = load_jailed_config()?;

            assert_eq!(
                config,
                WorkspaceConfig {
                    node: Some(NodeConfig::default()),
                    projects: HashMap::new(),
                    vcs: Some(VcsConfig::default()),
                }
            );

            Ok(())
        });
    }

    mod node {
        use super::*;

        #[test]
        fn loads_defaults() {
            figment::Jail::expect_with(|jail| {
                jail.create_file(
                    constants::CONFIG_WORKSPACE_FILENAME,
                    r#"
projects: {}
node:
    packageManager: yarn"#,
                )?;

                let config = super::load_jailed_config()?;

                assert_eq!(
                    config,
                    WorkspaceConfig {
                        node: Some(NodeConfig {
                            package_manager: Some(PackageManager::Yarn),
                            ..NodeConfig::default()
                        }),
                        projects: HashMap::new(),
                        vcs: Some(VcsConfig::default()),
                    }
                );

                Ok(())
            });
        }

        #[test]
        #[should_panic(
            expected = "Invalid field `node`. Expected struct NodeConfig type, received unsigned int `123`."
        )]
        fn invalid_type() {
            figment::Jail::expect_with(|jail| {
                jail.create_file(super::constants::CONFIG_WORKSPACE_FILENAME, "node: 123")?;

                super::load_jailed_config()?;

                Ok(())
            });
        }

        #[test]
        #[should_panic(
            expected = "Invalid field `node.version`. Must be a valid semantic version."
        )]
        fn invalid_version() {
            figment::Jail::expect_with(|jail| {
                jail.create_file(
                    super::constants::CONFIG_WORKSPACE_FILENAME,
                    r#"
node:
  version: 'foo bar'
projects:
  foo: packages/foo"#,
                )?;

                super::load_jailed_config()?;

                Ok(())
            });
        }

        #[test]
        #[should_panic(
            expected = "Invalid field `node.version`. Must be a valid semantic version."
        )]
        fn no_patch_version() {
            figment::Jail::expect_with(|jail| {
                jail.create_file(
                    super::constants::CONFIG_WORKSPACE_FILENAME,
                    r#"
node:
  version: '16.13'
projects:
  foo: packages/foo"#,
                )?;

                super::load_jailed_config()?;

                Ok(())
            });
        }

        #[test]
        #[should_panic(
            expected = "Invalid field `node.version`. Must be a valid semantic version."
        )]
        fn no_minor_version() {
            figment::Jail::expect_with(|jail| {
                jail.create_file(
                    super::constants::CONFIG_WORKSPACE_FILENAME,
                    r#"
node:
  version: '16'
projects:
  foo: packages/foo"#,
                )?;

                super::load_jailed_config()?;

                Ok(())
            });
        }

        #[test]
        #[should_panic(expected = "Invalid field `node.packageManager`. Unknown option `what`.")]
        fn invalid_package_manager() {
            figment::Jail::expect_with(|jail| {
                jail.create_file(
                    super::constants::CONFIG_WORKSPACE_FILENAME,
                    r#"
node:
  version: '16.13.0'
  packageManager: what
projects:
  foo: packages/foo"#,
                )?;

                super::load_jailed_config()?;

                Ok(())
            });
        }

        #[test]
        fn valid_package_manager() {
            figment::Jail::expect_with(|jail| {
                jail.create_file(
                    super::constants::CONFIG_WORKSPACE_FILENAME,
                    r#"
node:
  version: '16.13.0'
  packageManager: yarn
projects:
  foo: packages/foo"#,
                )?;

                super::load_jailed_config()?;

                Ok(())
            });
        }
    }

    mod npm {
        #[test]
        #[should_panic(
            expected = "Invalid field `node.npm`. Expected struct NpmConfig type, received string \"foo\"."
        )]
        fn invalid_type() {
            figment::Jail::expect_with(|jail| {
                jail.create_file(
                    super::constants::CONFIG_WORKSPACE_FILENAME,
                    r#"
node:
    version: '16.13.0'
    npm: foo"#,
                )?;

                super::load_jailed_config()?;

                Ok(())
            });
        }

        #[test]
        #[should_panic(
            expected = "Invalid field `node.npm.version`. Must be a valid semantic version."
        )]
        fn invalid_version() {
            figment::Jail::expect_with(|jail| {
                jail.create_file(
                    super::constants::CONFIG_WORKSPACE_FILENAME,
                    r#"
node:
    version: '16.13.0'
    npm:
        version: 'foo bar'
projects:
  foo: packages/foo"#,
                )?;

                super::load_jailed_config()?;

                Ok(())
            });
        }
    }

    mod pnpm {
        #[test]
        #[should_panic(
            expected = "Invalid field `node.pnpm`. Expected struct PnpmConfig type, received string \"foo\"."
        )]
        fn invalid_type() {
            figment::Jail::expect_with(|jail| {
                jail.create_file(
                    super::constants::CONFIG_WORKSPACE_FILENAME,
                    r#"
node:
    version: '16.13.0'
    pnpm: foo"#,
                )?;

                super::load_jailed_config()?;

                Ok(())
            });
        }

        #[test]
        #[should_panic(
            expected = "Invalid field `node.pnpm.version`. Must be a valid semantic version."
        )]
        fn invalid_version() {
            figment::Jail::expect_with(|jail| {
                jail.create_file(
                    super::constants::CONFIG_WORKSPACE_FILENAME,
                    r#"
node:
    version: '16.13.0'
    pnpm:
        version: 'foo bar'
projects:
  foo: packages/foo"#,
                )?;

                super::load_jailed_config()?;

                Ok(())
            });
        }
    }

    mod yarn {
        #[test]
        #[should_panic(
            expected = "Invalid field `node.yarn`. Expected struct YarnConfig type, received string \"foo\"."
        )]
        fn invalid_type() {
            figment::Jail::expect_with(|jail| {
                jail.create_file(
                    super::constants::CONFIG_WORKSPACE_FILENAME,
                    r#"
node:
    version: '16.13.0'
    yarn: foo"#,
                )?;

                super::load_jailed_config()?;

                Ok(())
            });
        }

        #[test]
        #[should_panic(
            expected = "Invalid field `node.yarn.version`. Must be a valid semantic version."
        )]
        fn invalid_version() {
            figment::Jail::expect_with(|jail| {
                jail.create_file(
                    super::constants::CONFIG_WORKSPACE_FILENAME,
                    r#"
node:
    version: '16.13.0'
    yarn:
        version: 'foo bar'
projects:
  foo: packages/foo"#,
                )?;

                super::load_jailed_config()?;

                Ok(())
            });
        }
    }

    mod projects {
        use std::collections::HashMap;

        #[test]
        #[should_panic(
            expected = "Invalid field `projects`. Expected a map type, received string \"apps/*\"."
        )]
        fn invalid_type() {
            figment::Jail::expect_with(|jail| {
                jail.create_file(
                    super::constants::CONFIG_WORKSPACE_FILENAME,
                    "projects: apps/*",
                )?;

                super::load_jailed_config()?;

                Ok(())
            });
        }

        #[test]
        #[should_panic(expected = "Invalid field `projects`. Absolute paths are not supported.")]
        fn no_abs_paths() {
            figment::Jail::expect_with(|jail| {
                jail.create_file(
                    super::constants::CONFIG_WORKSPACE_FILENAME,
                    r#"
projects:
  app: /apps/app
  foo: packages/foo"#,
                )?;

                super::load_jailed_config()?;

                Ok(())
            });
        }

        #[test]
        #[should_panic(
            expected = "Invalid field `projects`. Parent relative paths are not supported."
        )]
        fn no_parent_paths() {
            figment::Jail::expect_with(|jail| {
                jail.create_file(
                    super::constants::CONFIG_WORKSPACE_FILENAME,
                    r#"
projects:
  app: ../apps/app
  foo: packages/foo"#,
                )?;

                super::load_jailed_config()?;

                Ok(())
            });
        }

        #[test]
        fn valid_list() {
            figment::Jail::expect_with(|jail| {
                jail.create_file(
                    super::constants::CONFIG_WORKSPACE_FILENAME,
                    r#"
projects:
  app: apps/app
  foo: ./packages/foo"#,
                )?;

                let config = super::load_jailed_config()?;

                assert_eq!(
                    config.projects,
                    HashMap::from([
                        (String::from("app"), String::from("apps/app")),
                        (String::from("foo"), String::from("./packages/foo"))
                    ]),
                );

                Ok(())
            });
        }
    }

    mod vcs {
        use super::*;

        #[test]
        fn loads_defaults() {
            figment::Jail::expect_with(|jail| {
                jail.create_file(
                    constants::CONFIG_WORKSPACE_FILENAME,
                    r#"
projects: {}
vcs:
    manager: svn"#,
                )?;

                let config = super::load_jailed_config()?;

                assert_eq!(
                    config,
                    WorkspaceConfig {
                        node: Some(NodeConfig::default()),
                        projects: HashMap::new(),
                        vcs: Some(VcsConfig {
                            manager: Some(VcsManager::Svn),
                            ..VcsConfig::default()
                        }),
                    }
                );

                Ok(())
            });
        }

        #[test]
        #[should_panic(
            expected = "Invalid field `vcs`. Expected struct VcsConfig type, received unsigned int `123`."
        )]
        fn invalid_type() {
            figment::Jail::expect_with(|jail| {
                jail.create_file(
                    super::constants::CONFIG_WORKSPACE_FILENAME,
                    r#"
projects: {}
vcs: 123"#,
                )?;

                super::load_jailed_config()?;

                Ok(())
            });
        }

        #[test]
        #[should_panic(expected = "Invalid field `vcs.manager`. Unknown option `unknown`.")]
        fn invalid_manager_option() {
            figment::Jail::expect_with(|jail| {
                jail.create_file(
                    super::constants::CONFIG_WORKSPACE_FILENAME,
                    r#"
projects: {}
vcs:
    manager: unknown"#,
                )?;

                super::load_jailed_config()?;

                Ok(())
            });
        }

        #[test]
        #[should_panic(
            expected = "Invalid field `vcs.defaultBranch`. Expected a string type, received unsigned int `123`."
        )]
        fn invalid_default_branch_type() {
            figment::Jail::expect_with(|jail| {
                jail.create_file(
                    super::constants::CONFIG_WORKSPACE_FILENAME,
                    r#"
projects: {}
vcs:
    defaultBranch: 123"#,
                )?;

                super::load_jailed_config()?;

                Ok(())
            });
        }
    }
}