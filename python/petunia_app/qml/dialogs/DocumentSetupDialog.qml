import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Dialog {
    id: docSetupDialog
    title: "Configuração do Documento"
    modal: true
    anchors.centerIn: parent
    width: 340
    standardButtons: Dialog.Ok | Dialog.Cancel

    ColumnLayout {
        anchors.fill: parent
        spacing: 10

        RowLayout {
            spacing: 8
            Text { text: "Nome:"; color: "#a1a1aa"; Layout.preferredWidth: 70; font.pixelSize: 11 }
            TextField {
                text: bridge.documentName
                font.pixelSize: 11
                Layout.fillWidth: true
                onEditingFinished: bridge.setDocumentName(text)
            }
        }

        RowLayout {
            spacing: 8
            Text { text: "Largura:"; color: "#a1a1aa"; Layout.preferredWidth: 70; font.pixelSize: 11 }
            TextField {
                text: Math.round(bridge.documentWidth)
                font.pixelSize: 11
                Layout.fillWidth: true
                onEditingFinished: bridge.setDocumentSize(parseFloat(text) || 1200, bridge.documentHeight)
            }
            Text { text: "px"; color: "#71717a"; font.pixelSize: 10 }
        }

        RowLayout {
            spacing: 8
            Text { text: "Altura:"; color: "#a1a1aa"; Layout.preferredWidth: 70; font.pixelSize: 11 }
            TextField {
                text: Math.round(bridge.documentHeight)
                font.pixelSize: 11
                Layout.fillWidth: true
                onEditingFinished: bridge.setDocumentSize(bridge.documentWidth, parseFloat(text) || 800)
            }
            Text { text: "px"; color: "#71717a"; font.pixelSize: 10 }
        }

        RowLayout {
            spacing: 8
            Text { text: "DPI:"; color: "#a1a1aa"; Layout.preferredWidth: 70; font.pixelSize: 11 }
            ComboBox {
                model: ["72 DPI (Tela)", "150 DPI (Média)", "300 DPI (Impressão)"]
                currentIndex: 0
                Layout.fillWidth: true
            }
        }

        RowLayout {
            spacing: 8
            Text { text: "Cor:"; color: "#a1a1aa"; Layout.preferredWidth: 70; font.pixelSize: 11 }
            ComboBox {
                model: ["sRGB (Web / Digital)", "CMYK (Impressão Gráfica)", "Escala de Cinza"]
                currentIndex: 0
                Layout.fillWidth: true
            }
        }
    }
}
