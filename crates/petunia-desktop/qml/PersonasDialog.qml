pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Dialog {
    id: dialog
    required property StudioTheme theme
    required property var backend
    required property var studioState
    property string selectedId: ""
    ListModel { id: personaModel; dynamicRoles: true }
    function syncPersonas() {
        if (!personaModel) return
        var profiles = (studioState.preferences || {}).personas || []
        for (var i=0;i<profiles.length;i++) {
            if (i >= personaModel.count || personaModel.get(i).entry.id !== profiles[i].id) {
                var found=-1
                for (var j=i+1;j<personaModel.count;j++) if (personaModel.get(j).entry.id === profiles[i].id) { found=j; break }
                if (found >= 0) personaModel.move(found,i,1)
                else personaModel.insert(i,{entry:profiles[i]})
            }
            personaModel.setProperty(i,"entry",profiles[i])
        }
        if (personaModel.count > profiles.length) personaModel.remove(profiles.length,personaModel.count-profiles.length)
    }
    onStudioStateChanged: syncPersonas()
    Component.onCompleted: syncPersonas()
    property var editingTools: []
    property var editingPanels: []
    property var selectedPersona: {
        var personas = (studioState.preferences || {}).personas || []
        for (var i=0;i<personas.length;i++) if (personas[i].id === selectedId) return personas[i]
        return ({})
    }
    title: "Gerenciar personas"
    modal: true; width: Math.min(680*theme.uiScale, parent ? parent.width-32 : 680)
    height: Math.min(implicitHeight,parent ? parent.height-32 : implicitHeight)
    x: parent ? (parent.width-width)/2 : 0; y: parent ? (parent.height-height)/2 : 0
    standardButtons: Dialog.Close
    function operation(name, payload) {
        return backend.invoke("persona", JSON.stringify(Object.assign({operation:name,id:selectedId}, payload || {})))
    }
    onOpened: {
        selectedId = (studioState.preferences || {}).activePersona || "vector"
        nameField.text = selectedPersona.name || ""
    }
    onSelectedIdChanged: nameField.text = selectedPersona.name || ""
    contentItem: ScrollView {
        clip: true; contentWidth: availableWidth
        implicitHeight: dialogContent.implicitHeight
        ColumnLayout {
            id: dialogContent; width: parent.width
        spacing: 12
        Label {
            text: "Ativar muda o arranjo de trabalho. Mostrar apenas inclui a persona no seletor.";
            color: dialog.theme.secondary; wrapMode: Text.Wrap; Layout.fillWidth: true
        }
        ScrollView {
            Layout.fillWidth: true; Layout.preferredHeight: 240; clip: true
            ColumnLayout {
                width: parent.width; spacing: 4
                Repeater {
                    model: personaModel
                    RowLayout {
                        id: personaRow; required property var entry; readonly property var modelData: entry
                        Layout.fillWidth: true
                        StudioButton {
                            Layout.fillWidth: true; theme: dialog.theme; backend: dialog.backend
                            text: personaRow.modelData.name; iconKey: personaRow.modelData.icon || "persona.vector"
                            selectable: true; selected: personaRow.modelData.id === dialog.selectedId
                            onClicked: dialog.selectedId = personaRow.modelData.id
                        }
                        StudioButton {
                            theme: dialog.theme; text: personaRow.modelData.id === (dialog.studioState.preferences || {}).activePersona ? "Ativa" : "Ativar"
                            selectable: true; selected: personaRow.modelData.id === (dialog.studioState.preferences || {}).activePersona
                            onClicked: dialog.backend.invoke("persona", JSON.stringify({operation:"activate",id:personaRow.modelData.id}))
                        }
                        Switch {
                            checked: !!personaRow.modelData.visible
                            text: "Mostrar"
                            Accessible.name: "Mostrar persona " + personaRow.modelData.name
                            enabled: personaRow.modelData.id !== (dialog.studioState.preferences || {}).activePersona
                            ToolTip.visible: hovered; ToolTip.text: enabled ? "Mostrar no seletor" : "Ative outra persona antes de ocultar esta."
                            onClicked: {
                                var ok = dialog.backend.invoke("persona", JSON.stringify({operation:"show",id:personaRow.modelData.id,visible:checked}))
                                if (!ok) checked = !!personaRow.modelData.visible
                            }
                        }
                    }
                }
            }
        }
        Label { text: "Nome da persona selecionada"; color: dialog.theme.foreground }
        RowLayout {
            TextField { id: nameField; Layout.fillWidth: true; Accessible.name: "Nome da persona"; maximumLength: 64; selectByMouse: true }
            StudioButton { theme: dialog.theme; text: "Renomear"; enabled: nameField.text.trim().length > 0; onClicked: dialog.operation("rename", {name:nameField.text.trim()}) }
        }
        Flow {
            Layout.fillWidth: true; spacing: 4
            StudioButton { theme: dialog.theme; text: "Editar perfil…"; onClicked: {
                dialog.editingTools = (dialog.selectedPersona.tools || []).slice()
                dialog.editingPanels = (dialog.selectedPersona.panels || []).slice()
                editName.text = dialog.selectedPersona.name || ""
                editIcon.currentIndex = Math.max(0,editIcon.keys.indexOf(dialog.selectedPersona.icon || "persona.vector"))
                editDialog.open()
            } }
            StudioButton { theme: dialog.theme; text: "Duplicar"; onClicked: dialog.operation("duplicate",{name:(dialog.selectedPersona.name || "Persona") + " · cópia"}) }
            StudioButton { theme: dialog.theme; text: "Mover antes"; onClicked: dialog.operation("reorder",{direction:-1}) }
            StudioButton { theme: dialog.theme; text: "Mover depois"; onClicked: dialog.operation("reorder",{direction:1}) }
            StudioButton { theme: dialog.theme; text: "Restaurar arranjo…"; onClicked: restoreDialog.open() }
            StudioButton {
                theme: dialog.theme; text: "Excluir…"
                enabled: !dialog.selectedPersona.builtin && dialog.selectedId !== (dialog.studioState.preferences || {}).activePersona
                explanation: enabled ? "Excluir o perfil personalizado" : "Perfis originais e a persona ativa não podem ser excluídos."
                onClicked: removeDialog.open()
            }
        }
        Label {
            Layout.fillWidth: true; color: dialog.theme.secondary; wrapMode: Text.Wrap
            text: "Painéis: " + (dialog.selectedPersona.panels || []).join(", ") + "\nFerramentas: " + (dialog.selectedPersona.tools || []).join(", ")
        }
        Label { Layout.fillWidth: true; visible: !!dialog.studioState.error; text: dialog.studioState.error || ""; color: dialog.theme.error; wrapMode: Text.Wrap }
    }
    }
    Dialog {
        id: editDialog; title: "Editar persona"; modal: true
        parent: Overlay.overlay
        x: parent ? (parent.width-width)/2 : 0; y: parent ? (parent.height-height)/2 : 0
        height: Math.min(implicitHeight,parent ? parent.height-32 : implicitHeight)
        width: Math.min(500*dialog.theme.uiScale,dialog.width-24)
        standardButtons: Dialog.Apply | Dialog.Cancel
        onApplied: {
            if (dialog.operation("edit",{name:editName.text.trim(),icon:editIcon.keys[editIcon.currentIndex],tools:dialog.editingTools,panels:dialog.editingPanels})) close()
        }
        contentItem: ScrollView {
            clip: true; contentWidth: availableWidth; implicitHeight: editDialogContent.implicitHeight
        ColumnLayout {
            id: editDialogContent; width: parent.width
            Label { text: "Nome"; color: dialog.theme.foreground }
            TextField { id: editName; Layout.fillWidth: true; maximumLength: 64; Accessible.name: "Nome do perfil" }
            Label { text: "Ícone de atividade"; color: dialog.theme.foreground }
            ComboBox { id: editIcon; property var keys: ["persona.vector","persona.pixel","persona.layout"]; model: ["Vetor","Pixel","Layout"]; Accessible.name: "Ícone da persona"; Layout.fillWidth: true }
            Label { text: "Ferramentas disponíveis no perfil"; color: dialog.theme.foreground }
            Flow {
                Layout.fillWidth: true
                Repeater {
                    model: [{id:"tool.select",name:"Selecionar"},{id:"tool.node",name:"Editar vetor"},{id:"tool.pen",name:"Caneta"},{id:"tool.rectangle",name:"Retângulo"},{id:"tool.ellipse",name:"Elipse"},{id:"tool.zoom",name:"Zoom"}]
                    CheckBox {
                        required property var modelData
                        text: modelData.name; checked: dialog.editingTools.indexOf(modelData.id) >= 0
                        onClicked: {
                            var list = dialog.editingTools.filter(function(id) { return id !== modelData.id })
                            if (checked) list.push(modelData.id)
                            dialog.editingTools = list
                        }
                    }
                }
            }
            Label { text: "Painéis do perfil"; color: dialog.theme.foreground }
            Flow {
                Layout.fillWidth: true
                Repeater {
                    model: [{id:"layers",name:"Camadas"},{id:"inspector",name:"Inspetor"},{id:"color",name:"Cor"},{id:"history",name:"Histórico"}]
                    CheckBox {
                        required property var modelData
                        text: modelData.name; checked: dialog.editingPanels.indexOf(modelData.id) >= 0
                        onClicked: {
                            var list = dialog.editingPanels.filter(function(id) { return id !== modelData.id })
                            if (checked) list.push(modelData.id)
                            dialog.editingPanels = list
                        }
                    }
                }
            }
            Label { text: "Aplicar salva este arranjo. Cancelar preserva o perfil atual."; color: dialog.theme.secondary; wrapMode: Text.Wrap; Layout.fillWidth: true }
            Label { visible: !!dialog.studioState.error; text: dialog.studioState.error || ""; color: dialog.theme.error; wrapMode: Text.Wrap; Layout.fillWidth: true }
        }
        }
    }
    Dialog {
        id: removeDialog; title: "Excluir persona?"; modal: true
        parent: Overlay.overlay; width: Math.min(420*dialog.theme.uiScale,parent ? parent.width-32 : 420)
        x: parent ? (parent.width-width)/2 : 0; y: parent ? (parent.height-height)/2 : 0
        standardButtons: Dialog.Yes | Dialog.Cancel
        contentItem: Label { text: "Excluir “" + (dialog.selectedPersona.name || "") + "”? O documento será preservado."; wrapMode: Text.Wrap }
        onAccepted: dialog.operation("remove")
    }
    Dialog {
        id: restoreDialog; title: "Restaurar arranjo?"; modal: true
        parent: Overlay.overlay; width: Math.min(420*dialog.theme.uiScale,parent ? parent.width-32 : 420)
        x: parent ? (parent.width-width)/2 : 0; y: parent ? (parent.height-height)/2 : 0
        standardButtons: Dialog.Yes | Dialog.Cancel
        contentItem: Label { text: "Restaurar ferramentas e painéis de “" + (dialog.selectedPersona.name || "") + "”? As demais preferências serão preservadas."; wrapMode: Text.Wrap }
        onAccepted: dialog.operation("restore")
    }
}
