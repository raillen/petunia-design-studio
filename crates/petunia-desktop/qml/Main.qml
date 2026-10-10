pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs
import Petunia.Studio 1.0

ApplicationWindow {
    id: window
    objectName: "studioWindow"
    width: 1360; height: 860
    minimumWidth: 800; minimumHeight: 560
    visible: true
    title: (studioState.dirty ? "● " : "") + (studioState.title || "Sem título") + " · Petunia Design Studio"
    color: studioTheme.panel
    font.pixelSize: 14 * uiScale
    readonly property alias backend: studioBackend
    readonly property alias panelWorkspace: panels
    readonly property alias canvasView: canvas
    property real uiScale: (preferences || {}).uiScale || 1
    property int visiblePersonaLimit: width < 1100*uiScale ? 1 : 3
    property var studioState: {
        try { return JSON.parse(studioBackend.stateJson || "{}") } catch (error) { return ({status:"Não foi possível ler o estado da interface.",error:String(error)}) }
    }
    property var preferences: studioState.preferences || {}
    property string pendingAction: ""
    property string pendingPath: ""
    property bool allowClose: false
    property bool sidebarVisible: width >= 1024
    property var focusedItem: activeFocusItem
    property bool textEditing: (focusedItem && typeof focusedItem.cursorPosition !== "undefined") || (panels && panels.textEditing)
    property bool modalOpen: settings.visible || personas.visible || shortcutEditor.visible || paletteDialog.visible || dirtyDialog.visible || fileErrorDialog.visible || pendingEditDialog.visible
    property var currentPersona: {
        var list = (preferences || {}).personas || []
        for (var i=0;i<list.length;i++) if (list[i].id === preferences.activePersona) return list[i]
        return ({name:"Vetor",tools:["tool.select","tool.node","tool.pen","tool.rectangle","tool.ellipse","tool.zoom"]})
    }
    ListModel { id: topPersonasModel; dynamicRoles: true }
    ListModel { id: extraPersonasModel; dynamicRoles: true }
    function syncPersonaModel(model,list) {
        for (var i=0;i<list.length;i++) {
            if (i >= model.count || model.get(i).entry.id !== list[i].id) {
                var found=-1
                for (var j=i+1;j<model.count;j++) if (model.get(j).entry.id === list[i].id) { found=j; break }
                if (found >= 0) model.move(found,i,1)
                else model.insert(i,{entry:list[i]})
            }
            model.setProperty(i,"entry",list[i])
        }
        if (model.count > list.length) model.remove(list.length,model.count-list.length)
    }
    function syncPersonas() {
        if (!preferences || !topPersonasModel || !extraPersonasModel) return
        var profiles = (preferences.personas || []).filter(function(profile) { return profile.visible })
        syncPersonaModel(topPersonasModel,profiles.slice(0,visiblePersonaLimit))
        syncPersonaModel(extraPersonasModel,profiles.slice(visiblePersonaLimit))
    }
    onStudioStateChanged: syncPersonas()
    onVisiblePersonaLimitChanged: syncPersonas()
    Component.onCompleted: syncPersonas()
    StudioBackend { id: studioBackend }
    StudioTheme { id: studioTheme; appearance: window.preferences.appearance || "dark"; density: window.preferences.density || "comfortable"; uiScale: window.uiScale }
    palette.window: studioTheme.panel
    palette.windowText: studioTheme.foreground
    palette.base: studioTheme.raised
    palette.text: studioTheme.foreground
    palette.button: studioTheme.raised
    palette.buttonText: studioTheme.foreground
    palette.highlight: studioTheme.selected
    palette.highlightedText: studioTheme.selectedText
    palette.light: studioTheme.subtle
    palette.mid: studioTheme.controlBorder
    palette.dark: studioTheme.surround
    palette.toolTipBase: studioTheme.raised
    palette.toolTipText: studioTheme.foreground
    function invoke(command, payload) {
        var accepted = studioBackend.invoke(command, JSON.stringify(payload || {}))
        if (!accepted && (command === "save" || command === "saveAs" || command === "open" || command === "exportPng" || command === "new")) fileErrorDialog.open()
        return accepted
    }
    function focusRegion(reverse) {
        var regions = [personaRow,contextRow,toolRegion,canvas,panels]
        var item = activeFocusItem
        var index = -1
        while (item) {
            var current = regions.indexOf(item)
            if (current >= 0) { index = current; break }
            item = item.parent
        }
        for (var step=1;step<=regions.length;step++) {
            var next = regions[(index + (reverse ? -step : step) + regions.length*2) % regions.length]
            if (next.visible && (next.width > 0 || next === panels && panels.floating)) {
                if (next === panels && panels.floating) panels.focusContent()
                else { window.requestActivate(); next.forceActiveFocus(Qt.TabFocusReason) }
                break
            }
        }
    }
    function restoreFocus(item) {
        Qt.callLater(function() { if (!window.modalOpen && item && item.forceActiveFocus) item.forceActiveFocus() })
    }
    function qtShortcut(sequence) {
        return String(sequence).replace(/ArrowLeft/g,"Left").replace(/ArrowRight/g,"Right").replace(/ArrowUp/g,"Up").replace(/ArrowDown/g,"Down")
    }
    function shortcutFor(actionId) {
        var actions = studioState.actions || []
        for (var i=0;i<actions.length;i++) if (actions[i].id === actionId) return actions[i].shortcut || ""
        return ""
    }
    function prepareAction(action, path) {
        if (studioState.gestureActive || studioState.busy) { pendingEditDialog.open(); return }
        pendingAction = action; pendingPath = path || ""
        if (studioState.dirty) dirtyDialog.open()
        else continuePending()
    }
    function continuePending(discard) {
        var action = pendingAction; var path = pendingPath
        pendingAction = ""; pendingPath = ""
        if (action === "quit") { allowClose = true; close() }
        else if (action === "open") invoke("open",{path:path,discard:!!discard})
        else if (action === "new") invoke("new",{discard:!!discard})
    }
    function saveDocument(resume) {
        saveDialog.resumePending = !!resume
        if (studioState.path) {
            if (invoke("save",{path:studioState.path}) && resume) continuePending()
        } else saveDialog.open()
    }
    function dispatch(action, step) {
        if (action === "new" || action === "document.new") prepareAction("new")
        else if (action === "open" || action === "document.open") openDialog.open()
        else if (action === "save" || action === "document.save") saveDocument(false)
        else if (action === "saveAs" || action === "document.saveAs") { saveDialog.resumePending = false; saveDialog.open() }
        else if (action === "exportPng" || action === "document.export" || action === "document.exportPng") exportDialog.open()
        else if (action === "quit" || action === "application.quit") prepareAction("quit")
        else if (action === "preferences" || action === "workspace.preferences") settings.open()
        else if (action === "personas" || action === "workspace.personas") personas.open()
        else if (action === "shortcuts" || action === "workspace.shortcuts") shortcutEditor.open()
        else if (action === "palette" || action === "workspace.palette") paletteDialog.open()
        else if (action.indexOf("nudge.") === 0 && step) {
            invoke("nudge",{dx:action === "nudge.left" ? -step : action === "nudge.right" ? step : 0,dy:action === "nudge.up" ? -step : action === "nudge.down" ? step : 0})
        }
        else if (action === "workspace.focus_next") focusRegion(false)
        else if (action === "workspace.focus_previous") focusRegion(true)
        else if (action === "edit.cancel") { invoke("escape"); canvas.forceActiveFocus() }
        else invoke("command",{id:action})
    }
    onClosing: closeEvent => {
        if (!allowClose && (studioState.dirty || studioState.gestureActive || studioState.busy)) { closeEvent.accepted = false; prepareAction("quit") }
    }
    menuBar: MenuBar {
        Menu {
            title: "Arquivo"
            Action { text: "Novo"; onTriggered: window.prepareAction("new") }
            Action { text: "Abrir…"; onTriggered: openDialog.open() }
            MenuSeparator {}
            Action { text: "Salvar"; onTriggered: window.saveDocument(false) }
            Action { text: "Salvar como…"; onTriggered: { saveDialog.resumePending = false; saveDialog.open() } }
            Action { text: "Exportar PNG…"; onTriggered: exportDialog.open() }
            MenuSeparator {}
            Action { text: "Sair"; onTriggered: window.prepareAction("quit") }
        }
        Menu {
            title: "Editar"
            Action { text: "Desfazer"; enabled: !!window.studioState.canUndo; onTriggered: window.invoke("undo") }
            Action { text: "Refazer"; enabled: !!window.studioState.canRedo; onTriggered: window.invoke("redo") }
            MenuSeparator {}
            Action { text: "Selecionar tudo"; onTriggered: window.invoke("selectAll") }
            Action { text: "Duplicar"; enabled: window.studioState.selectionCount > 0; onTriggered: window.invoke("duplicate") }
            Action { text: "Excluir"; enabled: window.studioState.selectionCount > 0; onTriggered: window.invoke("delete") }
        }
        Menu {
            title: "Exibir"
            Action { text: "Enquadrar documento"; onTriggered: window.invoke("fit") }
            Action { text: "Painéis"; checkable: true; checked: window.sidebarVisible; onTriggered: window.sidebarVisible = !window.sidebarVisible }
            Menu {
                title: "Painéis do estúdio"
                Repeater {
                    model: [{id:"layers",label:"Camadas"},{id:"inspector",label:"Inspetor"},{id:"color",label:"Cor"},{id:"history",label:"Histórico"}]
                    MenuItem {
                        required property var modelData
                        text: modelData.label; checkable: true; checked: panels.panelOpen(modelData.id)
                        onTriggered: window.invoke("panel", {id:modelData.id,open:checked})
                    }
                }
            }
            Action { text: "Personalizar interface…"; onTriggered: settings.open() }
            Action { text: "Gerenciar personas…"; onTriggered: personas.open() }
        }
        Menu {
            title: "Comandos"
            Action { text: "Paleta de comandos…"; onTriggered: paletteDialog.open() }
            Action { text: "Editar atalhos…"; onTriggered: shortcutEditor.open() }
        }
    }
    header: ColumnLayout {
        spacing: 0
        Rectangle {
            Layout.fillWidth: true; implicitHeight: Math.max(52,personaRow.implicitHeight+12)
            color: studioTheme.panel
            border.width: personaRow.activeFocus ? 2 : 0; border.color: studioTheme.accent
            RowLayout {
                id: personaRow; activeFocusOnTab: true; Accessible.role: Accessible.ToolBar; Accessible.name: "Personas"; anchors.fill: parent; anchors.margins: 6; spacing: 8
                Label { visible: window.width >= 1100*window.uiScale; text: "PETUNIA"; color: studioTheme.foreground; font.bold: true; font.letterSpacing: 2; Layout.leftMargin: 8; Layout.rightMargin: 12 }
                Repeater {
                    model: topPersonasModel
                    StudioButton {
                        required property var entry
                        readonly property var modelData: entry
                        theme: studioTheme; backend: studioBackend; text: modelData.name
                        iconKey: modelData.icon || "persona.vector"
                        selectable: true; selected: modelData.id === window.preferences.activePersona
                        Accessible.description: selected ? "Persona ativa" : "Ativar persona"
                        onClicked: window.invoke("persona", {operation:"activate",id:modelData.id})
                    }
                }
                StudioButton {
                    visible: (window.preferences.personas || []).filter(function(p) { return p.visible }).length > window.visiblePersonaLimit
                    theme: studioTheme; text: {
                        var visibleProfiles = (window.preferences.personas || []).filter(function(p) { return p.visible })
                        return visibleProfiles.slice(0,window.visiblePersonaLimit).some(function(p) { return p.id === window.preferences.activePersona }) ? "Mais personas…" : (window.currentPersona.name || "Mais personas…")
                    }
                    selectable: true; selected: !(window.preferences.personas || []).filter(function(p) { return p.visible }).slice(0,window.visiblePersonaLimit).some(function(p) { return p.id === window.preferences.activePersona })
                    onClicked: overflowPersonas.open()
                    Menu {
                        id: overflowPersonas
                        Repeater {
                            model: extraPersonasModel
                            MenuItem {
                                required property var entry
                                readonly property var modelData: entry
                                text: modelData.name; checkable: true; checked: modelData.id === window.preferences.activePersona
                                onTriggered: window.invoke("persona",{operation:"activate",id:modelData.id})
                            }
                        }
                    }
                }
                StudioButton {
                    theme: studioTheme; backend: studioBackend; text: "Personas"; iconKey: "workspace.personas"
                    onClicked: personas.open()
                }
                Item { Layout.fillWidth: true }
                StudioButton { theme: studioTheme; backend: studioBackend; text: "Comandos"; iconOnly: window.width < 1100*window.uiScale; iconKey: "workspace.palette"; onClicked: paletteDialog.open() }
                StudioButton { theme: studioTheme; backend: studioBackend; text: "Personalizar"; iconOnly: window.width < 1100*window.uiScale; iconKey: "workspace.preferences"; onClicked: settings.open() }
            }
        }
        Rectangle {
            Layout.fillWidth: true; implicitHeight: Math.max(44,contextRow.implicitHeight+8)
            color: studioTheme.raised
            border.width: contextRow.activeFocus ? 2 : 0; border.color: studioTheme.accent
            RowLayout {
                id: contextRow; activeFocusOnTab: true; Accessible.role: Accessible.ToolBar; Accessible.name: "Barra contextual"; anchors.fill: parent; anchors.margins: 4; spacing: 8
                Label { text: window.studioState.context || "Seleção"; color: studioTheme.foreground; Layout.leftMargin: 12; Layout.fillWidth: true; elide: Text.ElideRight }
                Label { text: (window.studioState.selectionCount || 0) + " selecionado(s)"; color: studioTheme.secondary }
                StudioButton { theme: studioTheme; backend: studioBackend; text: "Confirmar"; iconKey: "action.confirm"; visible: !!window.studioState.gestureActive; onClicked: window.invoke("enter") }
                StudioButton { theme: studioTheme; backend: studioBackend; text: "Cancelar"; iconKey: "action.cancel"; visible: !!window.studioState.gestureActive; onClicked: window.invoke("escape") }
                StudioButton { theme: studioTheme; backend: studioBackend; text: "Desfazer"; iconKey: "edit.undo"; iconOnly: true; enabled: !!window.studioState.canUndo; shortcut: window.shortcutFor("edit.undo"); onClicked: window.invoke("undo") }
                StudioButton { theme: studioTheme; backend: studioBackend; text: "Refazer"; iconKey: "edit.redo"; iconOnly: true; enabled: !!window.studioState.canRedo; shortcut: window.shortcutFor("edit.redo"); onClicked: window.invoke("redo") }
                StudioButton { theme: studioTheme; backend: studioBackend; text: "Salvar"; iconKey: "document.save"; shortcut: window.shortcutFor("document.save"); onClicked: window.saveDocument(false) }
            }
        }
    }
    RowLayout {
        anchors.fill: parent; spacing: 1
        Rectangle {
            id: toolRegion; activeFocusOnTab: true
            Accessible.role: Accessible.ToolBar; Accessible.name: "Ferramentas"
            Layout.fillHeight: true; Layout.preferredWidth: studioTheme.target+16; color: studioTheme.panel
            border.width: activeFocus ? 2 : 0; border.color: studioTheme.accent
            ScrollView {
                anchors.fill: parent; anchors.margins: 8; clip: true
                ColumnLayout {
                    spacing: 4
                    Repeater {
                        model: [{tool:"select",name:"Selecionar"},{tool:"node",name:"Editar vetor"},{tool:"pen",name:"Caneta"},{tool:"rectangle",name:"Retângulo"},{tool:"ellipse",name:"Elipse"},{tool:"zoom",name:"Zoom"}]
                        StudioButton {
                            required property var modelData
                            theme: studioTheme; backend: studioBackend; iconOnly: true; text: modelData.name
                            visible: (window.currentPersona.tools || []).indexOf("tool." + modelData.tool) >= 0
                            iconKey: "tool." + modelData.tool
                            shortcut: window.shortcutFor("tool."+modelData.tool)
                            selectable: true; selected: window.studioState.tool === modelData.tool
                            onClicked: { window.invoke("chooseTool",{tool:modelData.tool}); canvas.forceActiveFocus() }
                        }
                    }
                    Rectangle { Layout.fillWidth: true; height: 1; color: studioTheme.subtle }
                    StudioButton { theme: studioTheme; backend: studioBackend; iconOnly: true; text: "Enquadrar documento"; iconKey: "view.fit"; onClicked: window.invoke("fit") }
                    StudioButton { theme: studioTheme; backend: studioBackend; iconOnly: true; text: "Mostrar painéis"; iconKey: "workspace.panels"; selectable: true; selected: window.sidebarVisible; onClicked: window.sidebarVisible = !window.sidebarVisible }
                }
            }
        }
        ColumnLayout {
            Layout.fillWidth: true; Layout.fillHeight: true; spacing: 0
            Rectangle {
                Layout.fillWidth: true; height: 24; color: studioTheme.panel
                Row {
                    anchors.fill: parent
                    Repeater {
                        model: Math.ceil(parent.width/100)
                        Label { required property int index; width: 100; height: 24; text: String(Math.round((index*100 - ((window.studioState.viewport || {}).panX || 0))/((window.studioState.viewport || {}).scale || 1))); color: studioTheme.secondary; font.pixelSize: 11; leftPadding: 4 }
                    }
                }
                Accessible.name: "Régua horizontal em unidades do documento"
            }
            CanvasView { id: canvas; objectName: "documentCanvas"; Layout.fillWidth: true; Layout.fillHeight: true; theme: studioTheme; backend: studioBackend; studioState: window.studioState; focus: true }
        }
        StudioPanels {
            id: panels; visible: window.sidebarVisible
            Layout.fillHeight: true; Layout.preferredWidth: floating ? 0 : Math.min(340*window.uiScale,window.width*0.35)
            theme: studioTheme; backend: studioBackend; studioState: window.studioState
            onColorRequested: { colorDialog.selectedColor = (window.studioState.selection || {}).fill || "#66D9EF"; colorDialog.open() }
            onFocusCanvas: canvas.forceActiveFocus()
        }
    }
    footer: Rectangle {
        color: studioTheme.panel; implicitHeight: Math.max(36,statusRow.implicitHeight+8)
        RowLayout {
            id: statusRow; anchors.fill: parent; anchors.margins: 4
            Label {
                id: statusLabel; property string lastAnnouncement: ""
                Layout.fillWidth: true; Layout.leftMargin: 8
                text: window.studioState.error || window.studioState.status || "Pronto"
                color: window.studioState.error ? studioTheme.error : studioTheme.secondary
                elide: Text.ElideRight
                Accessible.name: text
                ToolTip.visible: statusMouse.containsMouse; ToolTip.text: text
                MouseArea { id: statusMouse; anchors.fill: parent; hoverEnabled: true; acceptedButtons: Qt.NoButton }
            }
            Label { text: (window.studioState.documentWidth || 0) + " × " + (window.studioState.documentHeight || 0); color: studioTheme.secondary }
            StudioButton { theme: studioTheme; text: "−"; Accessible.name: "Diminuir zoom"; onClicked: window.invoke("zoom",{factor:1/1.2,anchorX:canvas.width/2,anchorY:canvas.height/2}) }
            Label { text: Math.round((window.studioState.zoom || 100)) + "%"; color: studioTheme.foreground }
            StudioButton { theme: studioTheme; text: "+"; Accessible.name: "Aumentar zoom"; onClicked: window.invoke("zoom",{factor:1.2,anchorX:canvas.width/2,anchorY:canvas.height/2}) }
        }
    }
    FileDialog {
        id: openDialog; title: "Abrir documento Petunia"; fileMode: FileDialog.OpenFile
        nameFilters: ["Documento Petunia (*.ptnd)"]
        onAccepted: window.prepareAction("open",String(selectedFile))
    }
    FileDialog {
        id: saveDialog; property bool resumePending: false
        title: "Salvar documento Petunia"; fileMode: FileDialog.SaveFile
        nameFilters: ["Documento Petunia (*.ptnd)"]; defaultSuffix: "ptnd"
        onAccepted: { if (window.invoke("saveAs",{path:String(selectedFile),overwrite:true}) && resumePending) window.continuePending() }
        onRejected: { if (resumePending) { window.pendingAction = ""; window.pendingPath = "" } }
    }
    FileDialog {
        id: exportDialog; title: "Exportar imagem PNG"; fileMode: FileDialog.SaveFile
        nameFilters: ["Imagem PNG (*.png)"]; defaultSuffix: "png"
        onAccepted: window.invoke("exportPng",{path:String(selectedFile),overwrite:true})
    }
    ColorDialog { id: colorDialog; title: "Escolher preenchimento"; onAccepted: window.invoke("setFill",{color:String(selectedColor)}) }
    Dialog {
        id: pendingEditDialog; title: "Edição em andamento"; modal: true
        parent: Overlay.overlay; width: Math.min(500*window.uiScale,parent.width-32)
        x: (parent.width-width)/2; y: (parent.height-height)/2
        standardButtons: Dialog.Ok
        contentItem: Label { text: "Confirme ou cancele a edição em andamento antes de criar, abrir outro documento ou sair. Seu trabalho permanece aberto."; color: studioTheme.foreground; wrapMode: Text.Wrap }
        onClosed: canvas.forceActiveFocus()
    }
    Dialog {
        id: fileErrorDialog; title: "Não foi possível concluir"; modal: true
        parent: Overlay.overlay; width: Math.min(500,parent.width-32)
        x: (parent.width-width)/2; y: (parent.height-height)/2
        standardButtons: Dialog.Ok
        contentItem: Label { text: window.studioState.error || "A operação falhou. Seu documento permanece aberto. Tente novamente após corrigir o problema."; color: studioTheme.error; wrapMode: Text.Wrap }
    }
    Dialog {
        id: dirtyDialog; title: "Salvar alterações?"; modal: true
        parent: Overlay.overlay
        x: (parent.width-width)/2; y: (parent.height-height)/2; width: Math.min(500,parent.width-32)
        closePolicy: Popup.CloseOnEscape
        onRejected: { window.pendingAction = ""; window.pendingPath = "" }
        contentItem: ColumnLayout {
            Label { text: "“" + (window.studioState.title || "Sem título") + "” contém alterações. Salve para preservá-las antes de continuar."; color: studioTheme.foreground; wrapMode: Text.Wrap; Layout.fillWidth: true }
            RowLayout {
                StudioButton { theme: studioTheme; text: "Salvar"; onClicked: { dirtyDialog.close(); window.saveDocument(true) } }
                StudioButton { theme: studioTheme; text: "Descartar"; onClicked: { dirtyDialog.close(); window.continuePending(true) } }
                StudioButton { theme: studioTheme; text: "Cancelar"; onClicked: dirtyDialog.reject() }
            }
        }
    }
    PreferencesDialog { id: settings; objectName: "preferencesDialog"; property var returnFocusItem; parent: Overlay.overlay; theme: studioTheme; backend: studioBackend; studioState: window.studioState; onAboutToShow: returnFocusItem = window.activeFocusItem; onClosed: window.restoreFocus(returnFocusItem) }
    PersonasDialog { id: personas; objectName: "personasDialog"; property var returnFocusItem; parent: Overlay.overlay; theme: studioTheme; backend: studioBackend; studioState: window.studioState; onAboutToShow: returnFocusItem = window.activeFocusItem; onClosed: window.restoreFocus(returnFocusItem) }
    ShortcutDialog { id: shortcutEditor; objectName: "shortcutDialog"; property var returnFocusItem; parent: Overlay.overlay; theme: studioTheme; backend: studioBackend; studioState: window.studioState; onAboutToShow: returnFocusItem = window.activeFocusItem; onClosed: window.restoreFocus(returnFocusItem) }
    CommandPalette { id: paletteDialog; objectName: "commandPalette"; property var returnFocusItem; parent: Overlay.overlay; theme: studioTheme; studioState: window.studioState; onActivated: actionId => window.dispatch(actionId); onAboutToShow: returnFocusItem = window.activeFocusItem; onClosed: window.restoreFocus(returnFocusItem) }
    Connections {
        target: studioBackend
        function onStateJsonChanged() {
            var message = window.studioState.error || window.studioState.status || ""
            if (!window.studioState.gestureActive && message && message !== statusLabel.lastAnnouncement) {
                statusLabel.lastAnnouncement = message
                if (typeof statusLabel.Accessible.announce === "function") statusLabel.Accessible.announce(message)
            }
        }
    }
    Repeater {
        model: window.studioState.actions || []
        Item {
            id: shortcutItem; required property var modelData
            Shortcut {
                sequence: window.qtShortcut(shortcutItem.modelData.shortcut || "")
                enabled: !!shortcutItem.modelData.shortcut && shortcutItem.modelData.enabled !== false && !window.modalOpen && (!window.textEditing || shortcutItem.modelData.id.indexOf("workspace.focus_") === 0)
                context: Qt.ApplicationShortcut
                onActivated: window.dispatch(shortcutItem.modelData.id)
            }
            Shortcut {
                sequence: shortcutItem.modelData.id.indexOf("nudge.") === 0 && shortcutItem.modelData.shortcut && String(shortcutItem.modelData.shortcut).indexOf("Shift+") < 0 ? "Shift+" + window.qtShortcut(shortcutItem.modelData.shortcut) : ""
                enabled: shortcutItem.modelData.id.indexOf("nudge.") === 0 && shortcutItem.modelData.enabled !== false && !window.modalOpen && !window.textEditing
                context: Qt.ApplicationShortcut
                onActivated: window.dispatch(shortcutItem.modelData.id,10)
            }
        }
    }
}
