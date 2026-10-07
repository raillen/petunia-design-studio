import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components"

Rectangle {
    id: quickFxPanel
    Layout.fillWidth: true
    Layout.fillHeight: true

    property color bgPanelBody: "#28282a"
    property color textPrimary: "#f2f2f5"
    property color textSecondary: "#a1a1a6"
    property color textDim: "#6c6c70"
    property color bgField: "#181819"
    property color borderSubtle: "#333335"

    property real blurRadius: 4.0
    property real shadowRadius: 8.0
    property real shadowOpacity: 0.5

    color: bgPanelBody

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 8
        spacing: 6

        Text {
            text: "Efeitos de Camada (Quick FX)"
            color: quickFxPanel.textPrimary
            font.pixelSize: 11
            font.bold: true
        }

        // Gaussian Blur Effect Row
        Rectangle {
            Layout.fillWidth: true
            height: 48
            radius: 3
            color: "#202022"
            border.color: "#333335"
            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 4
                spacing: 2
                RowLayout {
                    Layout.fillWidth: true
                    CheckBox {
                        id: blurCheck
                        text: "Desfoque Gaussiano"
                        contentItem: Text {
                            text: blurCheck.text
                            color: blurCheck.checked ? "#ffffff" : quickFxPanel.textSecondary
                            font.pixelSize: 10
                            font.bold: blurCheck.checked
                            leftPadding: 20
                        }
                    }
                    Item { Layout.fillWidth: true }
                    Text {
                        text: Math.round(blurSlider.value) + " px"
                        color: quickFxPanel.textSecondary
                        font.pixelSize: 9
                    }
                }
                PetuniaSlider {
                    id: blurSlider
                    Layout.fillWidth: true
                    enabled: blurCheck.checked
                    from: 0
                    to: 100
                    value: quickFxPanel.blurRadius
                    onMoved: quickFxPanel.blurRadius = value
                }
            }
        }

        // Outer Shadow Effect Row
        Rectangle {
            Layout.fillWidth: true
            height: 48
            radius: 3
            color: "#202022"
            border.color: "#333335"
            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 4
                spacing: 2
                RowLayout {
                    Layout.fillWidth: true
                    CheckBox {
                        id: shadowCheck
                        text: "Sombra Externa"
                        contentItem: Text {
                            text: shadowCheck.text
                            color: shadowCheck.checked ? "#ffffff" : quickFxPanel.textSecondary
                            font.pixelSize: 10
                            font.bold: shadowCheck.checked
                            leftPadding: 20
                        }
                    }
                    Item { Layout.fillWidth: true }
                    Text {
                        text: Math.round(shadowSlider.value) + " px"
                        color: quickFxPanel.textSecondary
                        font.pixelSize: 9
                    }
                }
                PetuniaSlider {
                    id: shadowSlider
                    Layout.fillWidth: true
                    enabled: shadowCheck.checked
                    from: 0
                    to: 50
                    value: quickFxPanel.shadowRadius
                    onMoved: quickFxPanel.shadowRadius = value
                }
            }
        }

        // Other quick FX checkboxes
        Repeater {
            model: ["Contorno (Outline)", "Brilho Interno (Inner Glow)", "Chanfro 3D (Bevel)", "Sobreposição de Cor"]
            delegate: Rectangle {
                Layout.fillWidth: true
                height: 26
                radius: 3
                color: "#202022"
                border.color: "#333335"
                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 4
                    anchors.rightMargin: 4
                    CheckBox {
                        text: modelData
                        contentItem: Text {
                            text: modelData
                            color: quickFxPanel.textSecondary
                            font.pixelSize: 10
                            leftPadding: 20
                        }
                    }
                }
            }
        }

        Item { Layout.fillHeight: true }
    }
}
