pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Dialog {
    id: dialog
    required property StudioTheme theme
    required property var backend
    required property var studioState
    property string actionId: ""
    property string actionLabel: ""
    property string conflict: {
        var actions = studioState.actions || []
        var value = shortcutField.text.trim().toLowerCase().replace(/\s/g, "")
        for (var i=0;i<actions.length;i++) {
            if (actions[i].id !== actionId && value && String(actions[i].shortcut || "").toLowerCase().replace(/\s/g, "") === value) return actions[i].label
        }
        return ""
    }
    title: "Atalhos de teclado"
    modal: true; width: Math.min(700*theme.uiScale, parent ? parent.width-32 : 700)
    height: Math.min(implicitHeight,parent ? parent.height-32 : implicitHeight)
    x: parent ? (parent.width-width)/2 : 0; y: parent ? (parent.height-height)/2 : 0
    standardButtons: Dialog.Close
    contentItem: ScrollView {
        clip: true; contentWidth: availableWidth
        implicitHeight: dialogContent.implicitHeight
        ColumnLayout {
            id: dialogContent; width: parent.width
        Label { text: "Selecione uma ação. Conflitos são verificados antes de aplicar."; color: dialog.theme.secondary; wrapMode: Text.Wrap; Layout.fillWidth: true }
        ScrollView {
            Layout.fillWidth: true; Layout.preferredHeight: 300; clip: true
            ColumnLayout {
                width: parent.width
                Repeater {
                    model: dialog.studioState.actions || []
                    StudioButton {
                        required property var modelData
                        Layout.fillWidth: true
                        theme: dialog.theme
                        text: modelData.label + "    " + (modelData.shortcut || "Sem atalho")
                        selectable: true; selected: modelData.id === dialog.actionId
                        enabled: modelData.rebindable !== false
                        explanation: modelData.rebindable === false ? "Esta ação ainda não oferece reatribuição de atalho." : ""
                        onClicked: {
                            dialog.actionId = modelData.id; dialog.actionLabel = modelData.label
                            shortcutField.text = modelData.shortcut || ""; shortcutField.forceActiveFocus()
                        }
                    }
                }
            }
        }
        Label { text: dialog.actionLabel || "Selecione uma ação"; color: dialog.theme.foreground }
        TextField {
            id: shortcutField; objectName: "shortcutEditor"; Layout.fillWidth: true
            Accessible.name: "Novo atalho"
            placeholderText: "Exemplo: Ctrl+Shift+P."
            enabled: !!dialog.actionId
            selectByMouse: true
        }
        Label { text: dialog.conflict ? "Este atalho já pertence a “" + dialog.conflict + "”." : ""; color: dialog.theme.error; Layout.fillWidth: true; wrapMode: Text.Wrap }
        RowLayout {
            StudioButton {
                theme: dialog.theme; text: "Aplicar atalho"; enabled: !!dialog.actionId && !!shortcutField.text.trim() && !dialog.conflict
                onClicked: dialog.backend.invoke("rebind",JSON.stringify({action:dialog.actionId,shortcut:shortcutField.text.trim()}))
            }
            StudioButton {
                theme: dialog.theme; text: "Restaurar padrão"; enabled: !!dialog.actionId
                onClicked: dialog.backend.invoke("rebind",JSON.stringify({action:dialog.actionId,restore:true}))
            }
        }
        Label { visible: !!dialog.studioState.error; text: dialog.studioState.error || ""; color: dialog.theme.error; wrapMode: Text.Wrap; Layout.fillWidth: true }
    }
    }
}
