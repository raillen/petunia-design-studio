import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components"

Rectangle {
    id: strokePanel
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
        spacing: 8

        // Espessura com PetuniaSlider
        RowLayout {
            Layout.fillWidth: true
            Text { text: "Espessura:"; color: strokePanel.textSecondary; font.pixelSize: 10; Layout.preferredWidth: 64 }
            PetuniaSlider {
                Layout.fillWidth: true
                from: 0
                to: 32
                value: bridge ? bridge.currentStrokeWidth : 1
                onMoved: if (bridge) bridge.setStrokeWidth(value)
            }
            Text {
                text: (bridge ? Math.round(bridge.currentStrokeWidth) : 1) + " pt"
                color: strokePanel.textPrimary
                font.pixelSize: 10
                Layout.preferredWidth: 32
            }
        }

        // Extremidade (Cap)
        RowLayout {
            Layout.fillWidth: true
            Text { text: "Extremidade:"; color: strokePanel.textSecondary; font.pixelSize: 10; Layout.preferredWidth: 64 }
            Repeater {
                model: [
                    { id: "round", label: "Redonda" },
                    { id: "butt", label: "Reta" },
                    { id: "square", label: "Quadrada" }
                ]
                delegate: Rectangle {
                    Layout.fillWidth: true
                    height: 22
                    radius: 3
                    color: (bridge && bridge.selectedStrokeCap === modelData.id) ? "#1976d2" : (capMouse.containsMouse ? "#333336" : "#242426")
                    border.color: (bridge && bridge.selectedStrokeCap === modelData.id) ? "#38bdf8" : "#38383b"
                    Text {
                        anchors.centerIn: parent
                        text: modelData.label
                        font.pixelSize: 9
                        color: "#ffffff"
                    }
                    MouseArea {
                        id: capMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: if (bridge) bridge.setStrokeCap(modelData.id)
                    }
                }
            }
        }

        // Junção (Join)
        RowLayout {
            Layout.fillWidth: true
            Text { text: "Junção:"; color: strokePanel.textSecondary; font.pixelSize: 10; Layout.preferredWidth: 64 }
            Repeater {
                model: [
                    { id: "round", label: "Redonda" },
                    { id: "miter", label: "Ângulo" },
                    { id: "bevel", label: "Chanfro" }
                ]
                delegate: Rectangle {
                    Layout.fillWidth: true
                    height: 22
                    radius: 3
                    color: (bridge && bridge.selectedStrokeJoin === modelData.id) ? "#1976d2" : (joinMouse.containsMouse ? "#333336" : "#242426")
                    border.color: (bridge && bridge.selectedStrokeJoin === modelData.id) ? "#38bdf8" : "#38383b"
                    Text {
                        anchors.centerIn: parent
                        text: modelData.label
                        font.pixelSize: 9
                        color: "#ffffff"
                    }
                    MouseArea {
                        id: joinMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: if (bridge) bridge.setStrokeJoin(modelData.id)
                    }
                }
            }
        }

        // Padrão / Traço
        RowLayout {
            Layout.fillWidth: true
            Text { text: "Padrão:"; color: strokePanel.textSecondary; font.pixelSize: 10; Layout.preferredWidth: 64 }
            Repeater {
                model: [
                    { id: "", label: "Contínuo" },
                    { id: "6, 4", label: "Tracejado" },
                    { id: "2, 3", label: "Pontilhado" }
                ]
                delegate: Rectangle {
                    Layout.fillWidth: true
                    height: 22
                    radius: 3
                    color: dashMouse.containsMouse ? "#333336" : "#242426"
                    border.color: "#38383b"
                    Text {
                        anchors.centerIn: parent
                        text: modelData.label
                        font.pixelSize: 9
                        color: "#ffffff"
                    }
                    MouseArea {
                        id: dashMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: if (bridge) bridge.setStrokeDash(modelData.id)
                    }
                }
            }
        }

        Item { Layout.fillHeight: true }
    }
}
