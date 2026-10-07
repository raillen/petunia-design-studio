import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components"

Rectangle {
    id: stylesPanel
    Layout.fillWidth: true
    Layout.fillHeight: true

    property color bgPanelBody: "#28282a"
    property color textPrimary: "#f2f2f5"
    property color textSecondary: "#a1a1a6"
    property color textDim: "#6c6c70"
    property color bgField: "#181819"
    property color borderSubtle: "#333335"

    color: bgPanelBody

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 8
        spacing: 6

        // Symbols Section
        Text {
            text: "Símbolos (" + (bridge ? bridge.symbols.length : 0) + ")"
            color: stylesPanel.textPrimary
            font.pixelSize: 10
            font.bold: true
        }

        ListView {
            Layout.fillWidth: true
            Layout.preferredHeight: 70
            model: bridge ? bridge.symbols : []
            clip: true
            delegate: RowLayout {
                width: ListView.view.width
                spacing: 4
                Text {
                    text: modelData.name
                    color: stylesPanel.textSecondary
                    font.pixelSize: 9
                    Layout.fillWidth: true
                }
                Button {
                    text: "Inserir"
                    font.pixelSize: 8
                    onClicked: if (bridge) bridge.placeSymbol(modelData.id, 100, 100)
                }
            }
        }

        Button {
            Layout.fillWidth: true
            text: "+ Criar Símbolo da Seleção"
            font.pixelSize: 9
            enabled: bridge && bridge.selectedId !== ""
            onClicked: if (bridge) bridge.createSymbol("", "")
        }

        // Styles Section
        Text {
            text: "Estilos Pré-definidos (" + (bridge ? bridge.styles.length : 0) + ")"
            color: stylesPanel.textPrimary
            font.pixelSize: 10
            font.bold: true
        }

        ListView {
            Layout.fillWidth: true
            Layout.preferredHeight: 70
            model: bridge ? bridge.styles : []
            clip: true
            delegate: RowLayout {
                width: ListView.view.width
                spacing: 4
                Text {
                    text: modelData.name
                    color: stylesPanel.textSecondary
                    font.pixelSize: 9
                    Layout.fillWidth: true
                }
                Button {
                    text: "Aplicar"
                    font.pixelSize: 8
                    onClicked: if (bridge) bridge.applyStyle(modelData.id)
                }
            }
        }

        Button {
            Layout.fillWidth: true
            text: "+ Criar Estilo da Seleção"
            font.pixelSize: 9
            enabled: bridge && bridge.selectedId !== ""
            onClicked: if (bridge) bridge.createStyle("Novo Estilo", "object", "")
        }

        Item { Layout.fillHeight: true }
    }
}
