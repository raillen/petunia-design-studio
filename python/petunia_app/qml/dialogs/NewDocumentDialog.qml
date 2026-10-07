import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components"

Dialog {
    id: newDocDialog
    title: "Novo Documento"
    modal: true
    anchors.centerIn: parent
    width: 460
    standardButtons: Dialog.Ok | Dialog.Cancel

    property string selectedPreset: "fhd"
    property real customWidth: 1920
    property real customHeight: 1080
    property string docTitle: "Sem título"

    onAccepted: {
        bridge.newDocument()
        bridge.setDocumentName(docTitle)
        bridge.setDocumentSize(customWidth, customHeight)
        bridge.createArtboard(docTitle, 0, 0, customWidth, customHeight, "#ffffff")
        bridge.fitAllArtboards()
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 12

        // Category Pills
        RowLayout {
            spacing: 6
            Repeater {
                model: [
                    { id: "web", name: "Web" },
                    { id: "print", name: "Impressão" },
                    { id: "devices", name: "Dispositivos" },
                    { id: "social", name: "Redes Sociais" }
                ]
                delegate: Button {
                    text: modelData.name
                    font.pixelSize: 10
                    flat: true
                    highlighted: catIndex === index
                    property int catIndex: 0
                    onClicked: catIndex = index
                }
            }
        }

        // Preset Grid
        GridLayout {
            columns: 2
            Layout.fillWidth: true
            rowSpacing: 6
            columnSpacing: 6

            Repeater {
                model: [
                    { name: "Full HD (1920 × 1080)", w: 1920, h: 1080 },
                    { name: "4K UHD (3840 × 2160)", w: 3840, h: 2160 },
                    { name: "A4 Papel (1240 × 1754 px)", w: 1240, h: 1754 },
                    { name: "Instagram Post (1080 × 1080)", w: 1080, h: 1080 },
                    { name: "Instagram Story (1080 × 1920)", w: 1080, h: 1920 },
                    { name: "Mobile UI (393 × 852)", w: 393, h: 852 }
                ]

                delegate: Rectangle {
                    Layout.fillWidth: true
                    height: 38
                    radius: 4
                    color: (newDocDialog.customWidth === modelData.w && newDocDialog.customHeight === modelData.h) ? "#3b82f6" : "#27272a"
                    border.color: "#3f3f46"

                    ColumnLayout {
                        anchors.centerIn: parent
                        spacing: 2
                        Text {
                            text: modelData.name
                            color: "#ffffff"
                            font.pixelSize: 10
                            font.bold: true
                        }
                    }

                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            newDocDialog.customWidth = modelData.w
                            newDocDialog.customHeight = modelData.h
                            newDocDialog.docTitle = modelData.name.split(" (")[0]
                        }
                    }
                }
            }
        }

        // Custom inputs
        Rectangle { Layout.fillWidth: true; height: 1; color: "#3f3f46" }

        RowLayout {
            spacing: 8
            Text { text: "Nome:"; color: "#a1a1aa"; font.pixelSize: 11; Layout.preferredWidth: 60 }
            TextField {
                text: newDocDialog.docTitle
                font.pixelSize: 11
                Layout.fillWidth: true
                onTextChanged: newDocDialog.docTitle = text
            }
        }

        RowLayout {
            spacing: 8
            Text { text: "Dimensões:"; color: "#a1a1aa"; font.pixelSize: 11; Layout.preferredWidth: 60 }
            TextField {
                text: Math.round(newDocDialog.customWidth)
                font.pixelSize: 11
                Layout.preferredWidth: 80
                onEditingFinished: newDocDialog.customWidth = parseFloat(text) || 1920
            }
            Text { text: "×"; color: "#a1a1aa"; font.pixelSize: 11 }
            TextField {
                text: Math.round(newDocDialog.customHeight)
                font.pixelSize: 11
                Layout.preferredWidth: 80
                onEditingFinished: newDocDialog.customHeight = parseFloat(text) || 1080
            }
            Text { text: "px"; color: "#71717a"; font.pixelSize: 10 }

            Button {
                contentItem: RowLayout {
                    spacing: 4
                    PIcon {
                        name: Phosphor.arrowsLeftRight
                        size: 11
                        color: "#ffffff"
                    }
                    Text {
                        text: "Inverter"
                        color: "#ffffff"
                        font.pixelSize: 10
                    }
                }
                onClicked: {
                    var tmp = newDocDialog.customWidth
                    newDocDialog.customWidth = newDocDialog.customHeight
                    newDocDialog.customHeight = tmp
                }
            }
        }
    }
}
