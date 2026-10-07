import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components"

Rectangle {
    id: swatchesPanel
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

        // Header: Title + Add Button
        RowLayout {
            Layout.fillWidth: true
            Text {
                text: "Amostras (" + (bridge ? bridge.swatches.length : 0) + ")"
                color: swatchesPanel.textPrimary
                font.pixelSize: 11
                font.bold: true
            }
            Item { Layout.fillWidth: true }
            Rectangle {
                width: 76
                height: 20
                radius: 3
                color: addSwatchMouse.containsMouse ? "#3f3f46" : "#27272a"
                border.color: "#3f3f46"
                RowLayout {
                    anchors.centerIn: parent
                    spacing: 3
                    Text { text: Phosphor.plus; font.family: "Phosphor"; font.pixelSize: 9; color: "#ffffff" }
                    Text { text: "Adicionar"; font.pixelSize: 9; color: "#ffffff" }
                }
                MouseArea {
                    id: addSwatchMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        if (bridge) bridge.addSwatch(bridge.currentFill)
                    }
                }
            }
        }

        // Swatches ScrollView
        ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true

            Flow {
                width: parent.width
                spacing: 4

                Repeater {
                    model: bridge ? bridge.swatches : []
                    delegate: Rectangle {
                        width: 22
                        height: 22
                        radius: 3
                        color: modelData
                        border.color: (bridge && bridge.currentFill.toUpperCase() === modelData.toUpperCase()) ? "#38bdf8" : "#3f3f46"
                        border.width: (bridge && bridge.currentFill.toUpperCase() === modelData.toUpperCase()) ? 2 : 1

                        MouseArea {
                            id: swatchMa
                            anchors.fill: parent
                            hoverEnabled: true
                            acceptedButtons: Qt.LeftButton | Qt.RightButton
                            cursorShape: Qt.PointingHandCursor
                            onClicked: function(mouse) {
                                if (mouse.button === Qt.RightButton) {
                                    if (bridge) bridge.removeSwatch(modelData)
                                } else {
                                    if (bridge) bridge.setFillColor(modelData)
                                }
                            }
                        }

                        ToolTip.visible: swatchMa.containsMouse
                        ToolTip.text: modelData + " (Botão direito para remover)"
                    }
                }
            }
        }
    }
}
