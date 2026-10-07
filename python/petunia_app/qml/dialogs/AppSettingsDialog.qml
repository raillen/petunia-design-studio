import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Dialog {
    id: appSettingsDialog
    title: "Configurações do Aplicativo"
    modal: true
    anchors.centerIn: parent
    width: 340
    standardButtons: Dialog.Close

    ColumnLayout {
        anchors.fill: parent
        spacing: 12

        CheckBox {
            text: "Snap to Grid (Grade de 8px)"
            checked: bridge.snappingEnabled
            onToggled: bridge.toggleSnapping()
        }

        CheckBox {
            text: "Auto-seleção de Objetos e Grupos"
            checked: bridge.autoSelectEnabled
            onToggled: bridge.setAutoSelectEnabled(checked)
        }

        CheckBox {
            text: "Renderização com Antialiasing de Alta Qualidade"
            checked: true
        }

        Rectangle { Layout.fillWidth: true; height: 1; color: "#3f3f46" }

        Button {
            text: "Carregar Documento Demo (Inioluwa Abiri)"
            Layout.fillWidth: true
            font.pixelSize: 11
            onClicked: {
                bridge.loadDemoDocument()
                appSettingsDialog.close()
            }
        }
    }
}
