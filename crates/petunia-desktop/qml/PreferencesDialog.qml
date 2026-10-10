pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Dialog {
    id: dialog
    required property StudioTheme theme
    required property var backend
    required property var studioState
    property var draft: ({})
    title: "Personalizar a interface"
    modal: true
    width: Math.min(520*theme.uiScale, parent ? parent.width - 32 : 520)
    height: Math.min(implicitHeight,parent ? parent.height-32 : implicitHeight)
    x: parent ? (parent.width-width)/2 : 0; y: parent ? (parent.height-height)/2 : 0
    standardButtons: Dialog.Apply | Dialog.Cancel
    closePolicy: Popup.CloseOnEscape
    onOpened: {
        draft = Object.assign({}, studioState.preferences || {})
        appearance.currentIndex = Math.max(0, appearance.options.indexOf(draft.appearance || "dark"))
        family.currentIndex = Math.max(0, family.options.indexOf(draft.iconFamily || "phosphor"))
        style.currentIndex = Math.max(0, style.options.indexOf(draft.iconStyle || "outline"))
        density.currentIndex = Math.max(0, density.options.indexOf(draft.density || "comfortable"))
        motion.checked = !!draft.reducedMotion
        scaleControl.currentIndex = Math.max(0,Math.round(((draft.uiScale || 1)-1)/0.25))
        appearance.forceActiveFocus()
    }
    onApplied: {
        if (backend.invoke("preferences", JSON.stringify({appearance:appearance.options[appearance.currentIndex],
            iconFamily:family.options[family.currentIndex], iconStyle:style.options[style.currentIndex],
            density:density.options[density.currentIndex],reducedMotion:motion.checked,uiScale:1+scaleControl.currentIndex*0.25}))) close()
    }
    contentItem: ScrollView {
        clip: true; contentWidth: availableWidth
        implicitHeight: dialogContent.implicitHeight
        ColumnLayout {
            id: dialogContent; width: parent.width
        spacing: 12
        Label { text: "Estas preferências não alteram seu documento."; color: dialog.theme.secondary; wrapMode: Text.Wrap; Layout.fillWidth: true }
        GridLayout {
            columns: dialog.width < 460*dialog.theme.uiScale ? 1 : 2; columnSpacing: 16; rowSpacing: 12
            Label { text: "Aparência"; color: dialog.theme.foreground }
            ComboBox { id: appearance; objectName: "appearanceChoice"; property var options: ["light","dark","highContrast"]; model: ["Claro","Escuro","Alto contraste"]; Accessible.name: "Aparência"; Layout.fillWidth: true }
            Label { text: "Família de ícones"; color: dialog.theme.foreground }
            ComboBox { id: family; objectName: "iconFamilyChoice"; property var options: ["phosphor","tabler"]; model: ["Phosphor","Tabler"]; Accessible.name: "Família de ícones"; Layout.fillWidth: true }
            Label { text: "Estilo dos ícones"; color: dialog.theme.foreground }
            ComboBox { id: style; property var options: ["outline","fill"]; model: ["Contorno (outline)","Preenchido (fill)"]; Accessible.name: "Estilo dos ícones"; Layout.fillWidth: true }
            Label { text: "Densidade"; color: dialog.theme.foreground }
            ComboBox { id: density; property var options: ["comfortable","compact"]; model: ["Confortável","Compacta"]; Accessible.name: "Densidade"; Layout.fillWidth: true }
        }
        RowLayout {
            Label { text: "Escala da interface"; color: dialog.theme.foreground }
            ComboBox { id: scaleControl; objectName: "interfaceScale"; model: ["100%","125%","150%","175%","200%"]; Accessible.name: "Escala da interface"; Layout.fillWidth: true }
        }
        CheckBox { id: motion; text: "Reduzir movimento"; Accessible.name: text }
        Label {
            Layout.fillWidth: true
            text: "Tabler: símbolos sem variante preenchida mantêm o contorno da mesma família. Seleção é indicada pelo botão, independentemente do estilo."
            color: dialog.theme.secondary; wrapMode: Text.Wrap
        }
        RowLayout {
            Label { text: "Prévia"; color: dialog.theme.secondary }
            Repeater {
                model: ["tool.select","tool.pen","document.save"]
                Image {
                    required property string modelData
                    source: { var snapshot = dialog.backend.stateJson; return dialog.backend.iconSource(modelData) }
                    sourceSize.width: 24; sourceSize.height: 24
                    Layout.preferredWidth: 24; Layout.preferredHeight: 24
                    Accessible.ignored: true
                }
            }
            Label { text: "A família será aplicada ao confirmar."; color: dialog.theme.secondary; wrapMode: Text.Wrap; Layout.fillWidth: true }
        }
        Label { visible: !!dialog.studioState.error; text: dialog.studioState.error || ""; color: dialog.theme.error; wrapMode: Text.Wrap; Layout.fillWidth: true }
    }
    }
}
