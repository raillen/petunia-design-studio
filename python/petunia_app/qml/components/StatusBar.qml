import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: statusBar
    width: parent ? parent.width : 1280
    height: 22

    property color bgToolbar: "#202021"
    property color borderDark: "#181819"
    property color textPrimary: "#f2f2f5"
    property color textSecondary: "#a1a1a6"
    property color textDim: "#6c6c70"
    property color accentCyan: "#00b4d8"

    property string canvasHintText: "Arraste para selecionar. Clique para selecionar prancheta ou objeto. Segure Espaço ou Meio do mouse para Pan."

    color: bgToolbar
    border.color: borderDark

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: 8
        anchors.rightMargin: 12
        spacing: 12

        // Artboard Page Navigator: Prev / Next
        RowLayout {
            spacing: 4

            Rectangle {
                width: 16
                height: 16
                radius: 3
                color: prevPageHover.containsMouse ? "#333336" : "transparent"
                Text {
                    anchors.centerIn: parent
                    text: Phosphor.caretLeft
                    color: (bridge && bridge.artboardCount > 1) ? statusBar.textPrimary : statusBar.textDim
                    font.family: "Phosphor"
                    font.pixelSize: 11
                }
                MouseArea {
                    id: prevPageHover
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: (bridge && bridge.artboardCount > 1) ? Qt.PointingHandCursor : Qt.ArrowCursor
                    onClicked: if (bridge) bridge.prevArtboard()
                }
                ToolTip.visible: prevPageHover.containsMouse
                ToolTip.text: "Prancheta Anterior"
            }

            Text {
                text: (bridge ? (bridge.activeArtboardIndex + 1) : 1) + " of " + Math.max(1, bridge ? bridge.artboardCount : 1)
                color: statusBar.textSecondary
                font.pixelSize: 10
                font.bold: true
            }

            Rectangle {
                width: 16
                height: 16
                radius: 3
                color: nextPageHover.containsMouse ? "#333336" : "transparent"
                Text {
                    anchors.centerIn: parent
                    text: Phosphor.caretRight
                    color: (bridge && bridge.artboardCount > 1) ? statusBar.textPrimary : statusBar.textDim
                    font.family: "Phosphor"
                    font.pixelSize: 11
                }
                MouseArea {
                    id: nextPageHover
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: (bridge && bridge.artboardCount > 1) ? Qt.PointingHandCursor : Qt.ArrowCursor
                    onClicked: if (bridge) bridge.nextArtboard()
                }
                ToolTip.visible: nextPageHover.containsMouse
                ToolTip.text: "Próxima Prancheta"
            }

            // Active Artboard Badge
            Rectangle {
                height: 16
                Layout.preferredWidth: Math.max(38, activeArtboardLayout.implicitWidth + 10)
                radius: 3
                color: "#28282b"
                border.color: "#3e3e42"
                RowLayout {
                    id: activeArtboardLayout
                    anchors.centerIn: parent
                    spacing: 3
                    Text {
                        text: Phosphor.frameCorners
                        color: statusBar.accentCyan
                        font.family: "Phosphor"
                        font.pixelSize: 10
                    }
                    Text {
                        id: activeArtboardText
                        text: bridge ? bridge.activeArtboardName : "Document"
                        color: statusBar.accentCyan
                        font.pixelSize: 9
                        font.bold: true
                    }
                }
                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: if (bridge) bridge.focusArtboard(bridge.selectedId)
                }
            }
        }

        Rectangle { width: 1; height: 12; color: "#38383a" }

        // Contextual Guidance Hint Text
        Text {
            text: statusBar.canvasHintText
            color: statusBar.textDim
            font.pixelSize: 10
            Layout.fillWidth: true
            elide: Text.ElideRight
        }

        // Right Status Indicator
        Text {
            text: (bridge ? bridge.documentName : "") + " (" + ((bridge ? bridge.zoom : 1.0) * 100).toFixed(1) + "%)"
            color: statusBar.textSecondary
            font.pixelSize: 10
        }
    }
}
