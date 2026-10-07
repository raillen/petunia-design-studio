import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components"

Rectangle {
    id: appearancePanel
    Layout.fillWidth: true
    Layout.fillHeight: true

    property color bgPanelBody: "#28282a"
    property color bgField: "#181819"
    property color borderSubtle: "#333335"
    property color textPrimary: "#f2f2f5"
    property color textSecondary: "#a1a1a6"
    property color textDim: "#6c6c70"

    color: bgPanelBody

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 8
        spacing: 6

        Text {
            text: "Aparência da Seleção"
            color: appearancePanel.textPrimary
            font.pixelSize: 11
            font.bold: true
        }

        // Preenchimento
        Rectangle {
            Layout.fillWidth: true
            height: 28
            radius: 3
            color: "#202022"
            border.color: "#333335"
            RowLayout {
                anchors.fill: parent
                anchors.margins: 4
                spacing: 6
                Rectangle {
                    width: 16
                    height: 16
                    radius: 2
                    color: bridge ? bridge.currentFill : "#e120a5"
                    border.color: "#555555"
                }
                Text {
                    text: "Preenchimento: " + (bridge ? bridge.currentFill : "Nenhum")
                    color: appearancePanel.textPrimary
                    font.pixelSize: 10
                    Layout.fillWidth: true
                }
            }
        }

        // Traço
        Rectangle {
            Layout.fillWidth: true
            height: 28
            radius: 3
            color: "#202022"
            border.color: "#333335"
            RowLayout {
                anchors.fill: parent
                anchors.margins: 4
                spacing: 6
                Rectangle {
                    width: 16
                    height: 16
                    radius: 2
                    color: "transparent"
                    border.color: bridge ? bridge.currentStroke : "#333333"
                    border.width: 2
                }
                Text {
                    text: "Traço: " + (bridge ? bridge.currentStroke : "Nenhum") + " (" + (bridge ? Math.round(bridge.currentStrokeWidth) : 1) + " pt)"
                    color: appearancePanel.textPrimary
                    font.pixelSize: 10
                    Layout.fillWidth: true
                }
            }
        }

        // Opacidade & Mesclagem
        Rectangle {
            Layout.fillWidth: true
            height: 28
            radius: 3
            color: "#202022"
            border.color: "#333335"
            RowLayout {
                anchors.fill: parent
                anchors.margins: 4
                spacing: 6
                Text {
                    text: "Opacidade: " + (bridge ? Math.round(bridge.selectedOpacity * 100) : 100) + "%  |  Modo: " + (bridge ? (bridge.selectedBlendMode || "normal") : "normal")
                    color: appearancePanel.textSecondary
                    font.pixelSize: 10
                    Layout.fillWidth: true
                }
            }
        }

        Item { Layout.fillHeight: true }
    }
}
