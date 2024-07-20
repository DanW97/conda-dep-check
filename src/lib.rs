use glob::glob;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, io, path::PathBuf};
use yaml_rust2::YamlLoader;

// This gets sent as a REST thing to the dependency API if using yaml
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Manifest {
    resolved: HashMap<String, Entry>,
    name: String,
    #[serde(rename = "file")]
    env_file: EnvFile,
}

impl Manifest {
    // TODO determine how/ should I set the name for this env
    pub fn new(env_file: PathBuf) -> io::Result<Self> {
        let env_file = EnvFile::new(env_file.to_str().unwrap())?;
        Ok(Manifest {
            env_file,
            ..Default::default()
        })
    }

    // Read env file and create as many Entry instances as required
    pub fn parse_env_file(self) -> Self {
        let mut entries = HashMap::new();
        let content =
            fs::read_to_string(self.env_file.env_file.clone()).expect("Could not read yaml file.");
        let env_file_contents =
            YamlLoader::load_from_str(&content).expect("Unable to parse env file");
        // println!("{:?}", env_file_contents);
        let env_file_content = &env_file_contents[0];
        // Conda entries are located in the `dependencies` tag and are all but last entry if we treat it as a vector
        for package in env_file_content["dependencies"].as_vec().unwrap() {
            // If the package is a Conda package, then `as_str` will be Some, and None if
            // a PyPi package owing to the fact that this is a nested series of entries
            match package.as_str() {
                Some(package) => {
                    let split = package.split('=').collect::<Vec<&str>>();
                    let package_name = split[0];
                    let package_version = if split.len() > 1 {
                        &format!("@{}", split[1])
                    } else {
                        ""
                    };
                    let package_url = format!("pkg:conda/{package_name}{package_version}");
                    let entry = Entry::new(&package_url);
                    entries.insert(package_url, entry);
                }
                None => {
                    let pypi_entries = package["pip"].as_vec().unwrap();
                    for pypi_package in pypi_entries {
                        let pypi_package = pypi_package.as_str().unwrap();
                        let split = pypi_package.split("==").collect::<Vec<&str>>();
                        // Normalise package names
                        let package_name = split[0].to_lowercase().replace('_', "-");
                        let package_version = if split.len() > 1 {
                            &format!("@{}", split[1])
                        } else {
                            ""
                        };
                        let package_url = format!("pkg:pypi/{package_name}{package_version}");
                        let entry = Entry::new(&package_url);
                        entries.insert(package_url, entry);
                    }
                }
            }
        }

        Manifest {
            resolved: entries,
            env_file: self.env_file,
            ..Default::default()
        }
    }

    pub fn submit_dependency_graph(self) {}
}

// Field in the test json
#[derive(Serialize, Deserialize, Debug, Default)]

struct EnvFile {
    #[serde(rename = "source_location")]
    env_file: String,
}

impl EnvFile {
    fn new(env_file: &str) -> io::Result<Self> {
        // Ensure that said file exists
        if fs::metadata(env_file).is_err() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Error, the file {env_file} doesn't exist."),
            ));
        }

        Ok(EnvFile {
            env_file: env_file.to_string(),
        })
    }
}

pub fn discover_environment_file() -> io::Result<PathBuf> {
    // Try for some form of env(ironment).yml file
    if let Some(entry) = glob("**/env*.yml")
        .expect("Invalid glob pattern.")
        .flatten()
        .next()
    {
        return Ok(entry);
    }

    // Try for some form of env(ironment).yaml file
    if let Some(entry) = glob("**/env*.yaml")
        .expect("Invalid glob pattern.")
        .flatten()
        .next()
    {
        return Ok(entry);
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "Failed to find any files matching env(ironment).y(a)ml".to_string(),
    ))
}

// Each entry into the manifest
#[derive(Serialize, Deserialize, Debug)]

pub struct Entry {
    package_url: String,
    relationship: String,
    dependencies: Vec<String>,
}

impl Entry {
    fn new(package_url: &str) -> Self {
        let package_url = package_url.to_string();
        Entry {
            package_url,
            ..Default::default()
        }
    }
}

impl Default for Entry {
    fn default() -> Self {
        Self {
            package_url: Default::default(),
            relationship: "direct".to_owned(),
            dependencies: [].to_vec(),
        }
    }
}

#[cfg(test)]
mod test {

    use serde_json::{json, to_value, Value};

    use crate::{EnvFile, Manifest};

    #[test]
    fn test_env_file() {
        let cwd = std::env::current_dir().unwrap();
        let env_file_location = cwd.join("test/environment.yaml");
        let env_file = EnvFile::new(env_file_location.to_str().unwrap());
        assert!(env_file.is_ok());
    }

    #[test]
    fn manifest_from_path() {
        let cwd = std::env::current_dir().unwrap();
        let env_file_location = cwd.join("test/environment.yaml");
        let manifest = Manifest::new(env_file_location);

        assert!(manifest.is_ok())
    }

    #[test]
    fn test_manifest() {
        let cwd = std::env::current_dir().unwrap();
        let env_file_location = cwd.join("test/environment.yaml");
        let manifest = Manifest::new(env_file_location)
            .expect("The env file could not be read.")
            .parse_env_file();
        let expected = desired_output();
        assert_eq!(to_value(manifest).unwrap(), expected);
    }

    // Chuck this at the bottom so I never have to look at it again!
    fn desired_output() -> Value {
        json!({
                    "resolved": {
            "pkg:conda/python@3.8": {
                "package_url": "pkg:conda/python@3.8",
                "relationship": "direct",
                "dependencies": []
            },
            "pkg:conda/pytorch@1.10": {
                "package_url": "pkg:conda/pytorch@1.10",
                "relationship": "direct",
                "dependencies": []
            },
            "pkg:conda/torchvision": {
                "package_url": "pkg:conda/torchvision",
                "relationship": "direct",
                "dependencies": []
            },
            "pkg:conda/cudatoolkit@11.0": {
                "package_url": "pkg:conda/cudatoolkit@11.0",
                "relationship": "direct",
                "dependencies": []
            },
            "pkg:conda/pip": {
                "package_url": "pkg:conda/pip",
                "relationship": "direct",
                "dependencies": []
            },
            "pkg:pypi/pytorch-lightning@1.5.2": {
                "package_url": "pkg:pypi/pytorch-lightning@1.5.2",
                "relationship": "direct",
                "dependencies": []
            },
            "pkg:pypi/einops@0.3.2": {
                "package_url": "pkg:pypi/einops@0.3.2",
                "relationship": "direct",
                "dependencies": []
            },
            "pkg:pypi/kornia@0.6.1": {
                "package_url": "pkg:pypi/kornia@0.6.1",
                "relationship": "direct",
                "dependencies": []
            },
            "pkg:pypi/opencv-python@4.5.4.58": {
                "package_url": "pkg:pypi/opencv-python@4.5.4.58",
                "relationship": "direct",
                "dependencies": []
            },
            "pkg:pypi/matplotlib@3.5.0": {
                "package_url": "pkg:pypi/matplotlib@3.5.0",
                "relationship": "direct",
                "dependencies": []
            },
            "pkg:pypi/imageio@2.10.4": {
                "package_url": "pkg:pypi/imageio@2.10.4",
                "relationship": "direct",
                "dependencies": []
            },
            "pkg:pypi/imageio-ffmpeg@0.4.5": {
                "package_url": "pkg:pypi/imageio-ffmpeg@0.4.5",
                "relationship": "direct",
                "dependencies": []
            },
            "pkg:pypi/torch-optimizer@0.3.0": {
                "package_url": "pkg:pypi/torch-optimizer@0.3.0",
                "relationship": "direct",
                "dependencies": []
            },
            "pkg:pypi/setuptools@58.2.0": {
                "package_url": "pkg:pypi/setuptools@58.2.0",
                "relationship": "direct",
                "dependencies": []
            },
            "pkg:pypi/pymcubes@0.1.2": {
                "package_url": "pkg:pypi/pymcubes@0.1.2",
                "relationship": "direct",
                "dependencies": []
            },
            "pkg:pypi/pycollada@0.7.1": {
                "package_url": "pkg:pypi/pycollada@0.7.1",
                "relationship": "direct",
                "dependencies": []
            },
            "pkg:pypi/trimesh@3.9.1": {
                "package_url": "pkg:pypi/trimesh@3.9.1",
                "relationship": "direct",
                "dependencies": []
            },
            "pkg:pypi/pyglet@1.5.10": {
                "package_url": "pkg:pypi/pyglet@1.5.10",
                "relationship": "direct",
                "dependencies": []
            },
            "pkg:pypi/networkx@2.5": {
                "package_url": "pkg:pypi/networkx@2.5",
                "relationship": "direct",
                "dependencies": []
            },
            "pkg:pypi/plyfile@0.7.2": {
                "package_url": "pkg:pypi/plyfile@0.7.2",
                "relationship": "direct",
                "dependencies": []
            },
            "pkg:pypi/open3d@0.13.0": {
                "package_url": "pkg:pypi/open3d@0.13.0",
                "relationship": "direct",
                "dependencies": []
            },
            "pkg:pypi/configargparse@1.5.3": {
                "package_url": "pkg:pypi/configargparse@1.5.3",
                "relationship": "direct",
                "dependencies": []
            },
            "pkg:pypi/ninja": {
                "package_url": "pkg:pypi/ninja",
                "relationship": "direct",
                "dependencies": []
            }
        },
        "name": "",
        "file": {
            "source_location": std::env::current_dir().unwrap().join("test/environment.yaml").to_str()
        }
                })
    }
}
