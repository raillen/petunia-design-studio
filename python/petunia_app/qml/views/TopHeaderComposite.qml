import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components"

Column {
    id: topHeaderComposite
    spacing: 0
    width: parent ? parent.width : 1366

    TopHeaderBar {
        id: topHeaderBar
        width: parent.width
        onRequestHelp: bridge.requestHelp()
    }

    ContextToolbar {
        id: contextToolbar
        width: parent.width
        onRequestDocumentSetup: bridge.requestDocumentSetup()
        onRequestAppSettings: bridge.requestAppSettings()
        onRequestPlaceImage: bridge.requestPlaceImage()
    }
}
