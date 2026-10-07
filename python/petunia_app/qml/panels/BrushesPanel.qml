import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components"

Rectangle {
    id: brushesPanel
    Layout.fillWidth: true
    Layout.fillHeight: true

    property color bgPanelBody: "#28282a"
    property color textPrimary: "#f2f2f5"
    property color textSecondary: "#a1a1a6"
    property color textDim: "#6c6c70"
    property color accentBlue: "#1976d2"
    property int selectedBrushIndex: 0

    color: bgPanelBody

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 8
        spacing: 6

        RowLayout {
            Layout.fillWidth: true
            Text {
                text: "Pincéis Vetoriais e Raster"
                color: brushesPanel.textPrimary
                font.pixelSize: 11
                font.bold: true
            }
            Item { Layout.fillWidth: true }
            Text {
                text: Phosphor.faders
                font.family: "Phosphor"
                font.pixelSize: 12
                color: brushesPanel.textSecondary
            }
        }

        ListView {
            id: brushesList
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            model: [
                { name: "Solid Round", size: "2 pt", type: "Vetor Básico" },
                { name: "Calligraphy 5pt", size: "5 pt", type: "Caligrafia" },
                { name: "Inked Outline", size: "3 pt", type: "Caneta Nanquim" },
                { name: "Textured Chalk", size: "8 pt", type: "Giz Texturizado" },
                { name: "Watercolor Edge", size: "12 pt", type: "Aquarela Suave" },
                { name: "Acrylic Wash", size: "16 pt", type: "Acrílico" },
                { name: "Charcoal Soft", size: "10 pt", type: "Carvão Macio" }
            ]

            delegate: Rectangle {
                width: ListView.view.width
                height: 28
                radius: 3
                color: brushesPanel.selectedBrushIndex === index ? brushesPanel.accentBlue : (brushMouse.containsMouse ? "#333336" : "transparent")

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 6
                    anchors.rightMargin: 6
                    spacing: 8

                    Text {
                        text: Phosphor.paintBrush
                        font.family: "Phosphor"
                        font.pixelSize: 12
                        color: brushesPanel.selectedBrushIndex === index ? "#ffffff" : brushesPanel.textSecondary
                    }

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 0
                        Text {
                            text: modelData.name
                            font.pixelSize: 10
                            font.bold: brushesPanel.selectedBrushIndex === index
                            color: brushesPanel.selectedBrushIndex === index ? "#ffffff" : brushesPanel.textPrimary
                        }
                        Text {
                            text: modelData.type
                            font.pixelSize: 8
                            color: brushesPanel.selectedBrushIndex === index ? "#e0e7ff" : brushesPanel.textDim
                        }
                    }

                    Text {
                        text: modelData.size
                        font.pixelSize: 9
                        color: brushesPanel.selectedBrushIndex === index ? "#ffffff" : brushesPanel.textSecondary
                    }
                }

                MouseArea {
                    id: brushMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        brushesPanel.selectedBrushIndex = index
                        if (bridge) bridge.setTool("vector_brush")
                    }
                }
            }
        }
    }
}
