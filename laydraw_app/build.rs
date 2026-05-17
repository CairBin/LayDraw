use std::{cmp::Ordering, fs, path::Path};

fn main() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("laydraw_app must live inside the workspace");
    let framework_manifest = workspace.join("Cargo.toml");
    let framework_version = read_package_version(&framework_manifest)
        .unwrap_or_else(|| panic!("missing version in {}", framework_manifest.display()));
    let plugin_root = workspace.join("plugin_packages");
    println!("cargo:rerun-if-changed={}", framework_manifest.display());
    println!("cargo:rerun-if-changed={}", plugin_root.display());

    let Ok(entries) = fs::read_dir(&plugin_root) else {
        return;
    };

    for entry in entries.flatten() {
        let manifest = entry.path().join("laydraw-plugin.toml");
        if !manifest.exists() {
            continue;
        }
        println!("cargo:rerun-if-changed={}", manifest.display());
        let content = fs::read_to_string(&manifest)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", manifest.display()));
        let id = read_key(&content, "id").unwrap_or_else(|| {
            entry
                .file_name()
                .to_string_lossy()
                .trim()
                .to_owned()
        });
        let support = read_key(&content, "support")
            .or_else(|| read_key(&content, "supported_laydraw"))
            .or_else(|| read_key(&content, "laydraw_version"))
            .or_else(|| read_key(&content, "max_laydraw_version").map(|value| format!("<={value}")))
            .unwrap_or_else(|| {
                panic!(
                    "{} must declare support, supported_laydraw, laydraw_version, or max_laydraw_version",
                    manifest.display()
                )
            });

        if !version_req_matches(&framework_version, &support) {
            panic!(
                "plugin `{id}` supports LayDraw `{support}`, but host framework version is `{framework_version}`"
            );
        }
    }
}

fn read_package_version(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    read_key(&content, "version")
}

fn read_key(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((left, right)) = line.split_once('=') else {
            continue;
        };
        if left.trim() == key {
            return Some(trim_toml_string(right.trim()).to_owned());
        }
    }
    None
}

fn trim_toml_string(value: &str) -> &str {
    value.trim()
        .trim_matches('"')
        .trim_matches('\'')
        .split('#')
        .next()
        .unwrap_or_default()
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
}

fn version_req_matches(version: &str, requirement: &str) -> bool {
    requirement
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .all(|part| comparator_matches(version, part))
}

fn comparator_matches(version: &str, comparator: &str) -> bool {
    let operators = [">=", "<=", ">", "<", "="];
    let (operator, expected) = operators
        .iter()
        .find_map(|operator| comparator.strip_prefix(operator).map(|rest| (*operator, rest.trim())))
        .unwrap_or(("=", comparator.trim()));

    let ordering = compare_versions(version, expected);
    match operator {
        "=" => ordering == Ordering::Equal,
        ">=" => matches!(ordering, Ordering::Equal | Ordering::Greater),
        "<=" => matches!(ordering, Ordering::Equal | Ordering::Less),
        ">" => ordering == Ordering::Greater,
        "<" => ordering == Ordering::Less,
        _ => false,
    }
}

fn compare_versions(left: &str, right: &str) -> Ordering {
    let left = parse_version(left);
    let right = parse_version(right);
    left.cmp(&right)
}

fn parse_version(version: &str) -> [u64; 3] {
    let mut parsed = [0, 0, 0];
    for (index, part) in version.split('.').take(3).enumerate() {
        let numeric = part
            .chars()
            .take_while(|ch| ch.is_ascii_digit())
            .collect::<String>();
        parsed[index] = numeric.parse().unwrap_or(0);
    }
    parsed
}
