#[cfg(feature = "native")]
fn main() {
    use cxx_qt_build::{CxxQtBuilder, QmlModule};
    let mut module = QmlModule::new("Petunia.Studio");
    let mut files: Vec<_> = std::fs::read_dir("qml")
        .expect("the desktop QML source directory is required")
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "qml"))
        .collect();
    files.sort();
    for file in files {
        module = module.qml_file(file);
    }
    module = module.qml_file("native-tests/NativeSmoke.qml");
    CxxQtBuilder::new_qml_module(module)
        .include_dir("src")
        .qt_module("Qml")
        .qt_module("Quick")
        .qt_module("QuickControls2")
        .files(["src/bridge.rs"])
        .build();
}

#[cfg(not(feature = "native"))]
fn main() {}
