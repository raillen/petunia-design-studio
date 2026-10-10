#pragma once
#include <QGuiApplication>
#include <QEventLoop>
#include <QTimer>
#include <QQuickItem>
#include <QUrl>
#include <QQuickWindow>
#include <QString>

// Native diagnostic capture. The QML smoke suite supplies an explicit path.
// Qt owns all windows; no pointers are retained beyond this synchronous call.
namespace petunia::desktop {
inline bool captureStudioWindow(const QString &path, const QString &frameSource) {
    for (auto *window : QGuiApplication::topLevelWindows()) {
        if (window->objectName() != QStringLiteral("studioWindow")) continue;
        auto *studio = qobject_cast<QQuickWindow *>(window);
        if (studio == nullptr) return false;
        const auto *canvas = studio->findChild<QQuickItem *>(QStringLiteral("canvasFrame"));
        const auto marker = QStringLiteral("data:image/png;base64,");
        if (canvas == nullptr || !frameSource.startsWith(marker)) return false;
        const auto expected = QImage::fromData(QByteArray::fromBase64(frameSource.mid(marker.size()).toUtf8()), "PNG");
        if (expected.isNull()) return false;
        // Image.Ready only confirms decoding. Offscreen software rendering may
        // still hold an earlier scenegraph frame. Wait for actual pixel readback.
        for (int attempt = 0; attempt < 20; ++attempt) {
            studio->update();
            QEventLoop wait;
            QTimer::singleShot(50, &wait, &QEventLoop::quit);
            wait.exec();
            const auto image = studio->grabWindow();
            if (image.isNull()) continue;
            const auto currentSource = canvas->property("source").toUrl().toString();
            const auto currentFrame = QImage::fromData(QByteArray::fromBase64(currentSource.mid(marker.size()).toUtf8()), "PNG");
            if (currentFrame.isNull()) continue;
            int mismatches = 0;
            for (int row = 1; row < 10; ++row) {
                for (int column = 1; column < 10; ++column) {
                    const auto point = canvas->mapToScene(QPointF(canvas->width() * column / 10.0, canvas->height() * row / 10.0));
                    const auto pixel = QPoint(qRound(point.x() * image.width() / studio->width()),
                                              qRound(point.y() * image.height() / studio->height()));
                    if (!image.rect().contains(pixel)) return false;
                    const auto actual = image.pixelColor(pixel);
                    const auto wanted = currentFrame.pixelColor(currentFrame.width() * column / 10,
                                                               currentFrame.height() * row / 10);
                    if (qAbs(actual.red() - wanted.red()) > 8 ||
                        qAbs(actual.green() - wanted.green()) > 8 ||
                        qAbs(actual.blue() - wanted.blue()) > 8) ++mismatches;
                }
            }
            // A couple of edge samples may differ under fractional scaling.
            if (mismatches <= 2) {
                return image.save(path, "PNG");
            }
        }
        return false;
    }
    return false;
}
}
