#!/usr/bin/env python3
"""Verify independent and packaged consumers with the REQUIRED safe-source override.
No unsafe consumer reproducer runs outside Miri. Metadata checks prove resolution.
"""
import argparse, hashlib, json, pathlib, subprocess, tarfile, tempfile
root = pathlib.Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument('--packaged', action='store_true')
args = parser.parse_args()
with tempfile.TemporaryDirectory(prefix='jzon-safe-consumer-') as temporary:
    temp = pathlib.Path(temporary)
    paths = {name: root/'crates'/folder for name, folder in [('jzon-rs','jzon'),('jzon-rs-compat','jzon_compat'),('jzon-rs-serde','jzon_serde'),('jzon-rs-derive','jzon_derive')]}
    safe = root/'vendor/serde_json'
    if args.packaged:
        for name in paths:
            archive = root/'target/package'/f'{name}-0.4.0.crate'
            with tarfile.open(archive) as tar:
                tar.extractall(temp, filter='data')
            paths[name] = temp/f'{name}-0.4.0'
        with tarfile.open(root/'vendor/serde_json/target/package/serde_json-1.0.151.crate') as tar:
            tar.extractall(temp, filter='data')
        safe = temp/'serde_json-1.0.151'
    consumer = temp/'consumer'
    (consumer/'src').mkdir(parents=True)
    manifest = '''[package]
name = "safe-source-consumer"
version = "0.0.0"
edition = "2021"
[workspace]
[dependencies]
core_json = { package = "jzon-rs", version = "=0.4.0", features = ["compat"] }
facade = { package = "jzon-rs-compat", version = "=0.4.0" }
serde_json = "=1.0.151"
serde = { version = "1", features = ["derive"] }
[patch.crates-io]
'''
    for name, path in paths.items():
        manifest += f'{name} = {{ path = {json.dumps(str(path))} }}\n'
    manifest += f'serde_json = {{ path = {json.dumps(str(safe))} }}\n'
    (consumer/'Cargo.toml').write_text(manifest)
    (consumer/'src/main.rs').write_text('''use serde::Deserialize;
use std::collections::BTreeMap;
fn main() {
    let value: serde_json::Value = facade::json!({"ok":1});
    let same: core_json::compat::Value = value.clone();
    assert_eq!(facade::to_string(&same).unwrap(),serde_json::to_string(&value).unwrap());
    for input in [r#"{"é":1}"#, r#"{"😀":1}"#, r#"{"true":1,"é":2}"#] {
        assert!(facade::from_str::<BTreeMap<bool,u8>>(input).is_err());
        assert!(core_json::compat::from_str::<BTreeMap<bool,u8>>(input).is_err());
        let mut de=facade::Deserializer::from_str(input);
        assert!(BTreeMap::<bool,u8>::deserialize(&mut de).is_err());
        assert!(core_json::compat::Deserializer::from_str(input).into_iter::<BTreeMap<bool,u8>>().next().unwrap().is_err());
        assert!(facade::Deserializer::from_slice(input.as_bytes()).into_iter::<BTreeMap<bool,u8>>().next().unwrap().is_err());
        assert!(core_json::compat::Deserializer::from_reader(input.as_bytes()).into_iter::<BTreeMap<bool,u8>>().next().unwrap().is_err());
        assert!(facade::Deserializer::from_reader(input.as_bytes()).into_iter::<BTreeMap<bool,u8>>().next().unwrap().is_err());
        assert!(core_json::compat::from_reader::<_,BTreeMap<bool,u8>>(input.as_bytes()).is_err());
        assert!(facade::from_reader::<_,BTreeMap<bool,u8>>(input.as_bytes()).is_err());
    }
}
''')
    if args.packaged:
        # Metadata only: prove why registry-normalized artifacts require the override.
        unsafe_manifest=manifest.replace(f'serde_json = {{ path = {json.dumps(str(safe))} }}\n', '')
        (consumer/'Cargo.toml').write_text(unsafe_manifest)
        without=json.loads(subprocess.check_output(['cargo','metadata','--format-version','1','--manifest-path',str(consumer/'Cargo.toml')],text=True))
        registry_json=[p for p in without['packages'] if p['name']=='serde_json']
        assert len(registry_json)==1 and registry_json[0]['source'].startswith('registry+'), registry_json
        print('REGISTRY_RELEASE_BLOCKER without override:',registry_json[0]['id'],flush=True)
        (consumer/'Cargo.toml').write_text(manifest)
    meta=json.loads(subprocess.check_output(['cargo','metadata','--format-version','1','--manifest-path',str(consumer/'Cargo.toml')],text=True))
    packages=[p for p in meta['packages'] if p['name']=='serde_json']
    assert len(packages)==1, packages
    assert pathlib.Path(packages[0]['manifest_path']).parent.resolve()==safe.resolve(), packages
    expected=hashlib.sha256((root/'vendor/serde_json/src/de.rs').read_bytes()).hexdigest()
    assert hashlib.sha256((safe/'src/de.rs').read_bytes()).hexdigest()==expected
    print('VERIFIED safe source:', packages[0]['id'], expected, flush=True)
    subprocess.run(['cargo','+nightly','miri','run','--manifest-path',str(consumer/'Cargo.toml')],check=True)
