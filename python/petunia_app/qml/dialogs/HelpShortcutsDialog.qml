import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Dialog {
    id: helpDialog
    title: "Ajuda & Atalhos de Teclado"
    modal: true
    anchors.centerIn: parent
    width: 380
    standardButtons: Dialog.Close

    ColumnLayout {
        anchors.fill: parent
        spacing: 6

        Text {
            text: "Petunia Design Studio — Atalhos de Teclado"
            font.bold: true
            color: "#ffffff"
            font.pixelSize: 12
        }

        Rectangle { Layout.fillWidth: true; height: 1; color: "#3f3f46" }

        GridLayout {
            columns: 2
            Layout.fillWidth: true
            columnSpacing: 16
            rowSpacing: 4

            Text { text: "V"; color: "#38bdf8"; font.bold: true; font.pixelSize: 11 }
            Text { text: "Ferramenta Mover (Seleção)"; color: "#d4d4d8"; font.pixelSize: 11 }

            Text { text: "A"; color: "#38bdf8"; font.bold: true; font.pixelSize: 11 }
            Text { text: "Ferramenta de Prancheta (Artboard)"; color: "#d4d4d8"; font.pixelSize: 11 }

            Text { text: "N"; color: "#38bdf8"; font.bold: true; font.pixelSize: 11 }
            Text { text: "Ferramenta de Nó (Edição de Curvas)"; color: "#d4d4d8"; font.pixelSize: 11 }

            Text { text: "C"; color: "#38bdf8"; font.bold: true; font.pixelSize: 11 }
            Text { text: "Ferramenta de Cantos"; color: "#d4d4d8"; font.pixelSize: 11 }

            Text { text: "P"; color: "#38bdf8"; font.bold: true; font.pixelSize: 11 }
            Text { text: "Caneta Bézier (Pen Tool)"; color: "#d4d4d8"; font.pixelSize: 11 }

            Text { text: "B"; color: "#38bdf8"; font.bold: true; font.pixelSize: 11 }
            Text { text: "Lápis / Pincel Vetorial"; color: "#d4d4d8"; font.pixelSize: 11 }

            Text { text: "M / R"; color: "#38bdf8"; font.bold: true; font.pixelSize: 11 }
            Text { text: "Retângulo"; color: "#d4d4d8"; font.pixelSize: 11 }

            Text { text: "O"; color: "#38bdf8"; font.bold: true; font.pixelSize: 11 }
            Text { text: "Elipse"; color: "#d4d4d8"; font.pixelSize: 11 }

            Text { text: "T"; color: "#38bdf8"; font.bold: true; font.pixelSize: 11 }
            Text { text: "Texto de Moldura"; color: "#d4d4d8"; font.pixelSize: 11 }

            Text { text: "G"; color: "#38bdf8"; font.bold: true; font.pixelSize: 11 }
            Text { text: "Preenchimento / Gradiente"; color: "#d4d4d8"; font.pixelSize: 11 }

            Text { text: "Y"; color: "#38bdf8"; font.bold: true; font.pixelSize: 11 }
            Text { text: "Transparência"; color: "#d4d4d8"; font.pixelSize: 11 }

            Text { text: "I"; color: "#38bdf8"; font.bold: true; font.pixelSize: 11 }
            Text { text: "Conta-gotas (Color Picker)"; color: "#d4d4d8"; font.pixelSize: 11 }

            Text { text: "Espaço"; color: "#38bdf8"; font.bold: true; font.pixelSize: 11 }
            Text { text: "Arrastar para Panorâmica (Pan)"; color: "#d4d4d8"; font.pixelSize: 11 }

            Text { text: "Ctrl + 0"; color: "#38bdf8"; font.bold: true; font.pixelSize: 11 }
            Text { text: "Ajustar todas as pranchetas à tela"; color: "#d4d4d8"; font.pixelSize: 11 }

            Text { text: "Ctrl + Z / Shift+Z"; color: "#38bdf8"; font.bold: true; font.pixelSize: 11 }
            Text { text: "Desfazer / Refazer"; color: "#d4d4d8"; font.pixelSize: 11 }

            Text { text: "Ctrl + G / Shift+G"; color: "#38bdf8"; font.bold: true; font.pixelSize: 11 }
            Text { text: "Agrupar / Desagrupar Seleção"; color: "#d4d4d8"; font.pixelSize: 11 }
        }
    }
}
