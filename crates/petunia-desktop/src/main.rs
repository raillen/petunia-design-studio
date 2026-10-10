//! Native entry point. Qt remains confined to the desktop adapter.
#![deny(unsafe_code)]
use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QString, QUrl};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

fn main() {
    petunia_desktop::bridge::initialize_native();
    let mut application = QGuiApplication::new();
    application
        .pin_mut()
        .set_application_name(&QString::from("Petunia Design Studio"));
    application
        .pin_mut()
        .set_organization_name(&QString::from("Petunia"));
    let mut engine = QQmlApplicationEngine::new();
    let failed = Arc::new(AtomicBool::new(false));
    let failed_signal = Arc::clone(&failed);
    let connection = engine
        .pin_mut()
        .on_object_creation_failed(move |_, _| failed_signal.store(true, Ordering::Relaxed));
    let source = if std::env::args().any(|arg| arg == "--smoke-test") {
        "qrc:/qt/qml/Petunia/Studio/native-tests/NativeSmoke.qml"
    } else {
        "qrc:/qt/qml/Petunia/Studio/qml/Main.qml"
    };
    engine.pin_mut().load(&QUrl::from(source));
    let result = if failed.load(Ordering::Relaxed) {
        1
    } else {
        application.pin_mut().exec()
    };
    drop(connection);
    drop(engine);
    drop(application);
    std::process::exit(result);
}
