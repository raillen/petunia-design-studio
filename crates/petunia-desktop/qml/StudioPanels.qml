pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Window

Item {
    id: root
    activeFocusOnTab: true
    Accessible.role: Accessible.Pane
    Accessible.name: "Painéis do estúdio"
    required property StudioTheme theme
    required property var backend
    required property var studioState
    property var contentFocusItem: contentLoader.item
    readonly property alias panelContentItem: contentLoader.item
    onFloatingChanged: if (!floating && root.Window.window) root.Window.window.requestActivate()
    property bool floating: false
    property var floatingFocusItem: floatingWindow.activeFocusItem
    readonly property bool textEditing: floating && floatingFocusItem && typeof floatingFocusItem.cursorPosition !== "undefined"
    property var collapsedIds: []
    ListModel { id: layerModel; dynamicRoles: true }
    function syncLayers() {
        if (!layerModel) return
        var layers = studioState.layers || []
        for (var i=0;i<layers.length;i++) {
            if (i >= layerModel.count || layerModel.get(i).entry.id !== layers[i].id) {
                var found = -1
                for (var j=i+1;j<layerModel.count;j++) if (layerModel.get(j).entry.id === layers[i].id) { found=j; break }
                if (found >= 0) layerModel.move(found,i,1)
                else layerModel.insert(i,{entry:layers[i]})
            }
            layerModel.setProperty(i,"entry",layers[i])
        }
        if (layerModel.count > layers.length) layerModel.remove(layers.length,layerModel.count-layers.length)
    }
    onStudioStateChanged: syncLayers()
    Component.onCompleted: syncLayers()
    property var layerById: {
        var map = {}
        ;(studioState.layers || []).forEach(function(layer) { map[layer.id] = layer })
        return map
    }
    function layerVisible(layer) {
        var ancestor = layer.parent
        for (var count=0;ancestor && count<100;count++) {
            if (collapsedIds.indexOf(ancestor) >= 0) return false
            ancestor = layerById[ancestor] ? layerById[ancestor].parent : null
        }
        return true
    }
    function toggleChildren(id) {
        var collapsed = collapsedIds.filter(function(value) { return value !== id })
        if (collapsedIds.indexOf(id) < 0) collapsed.push(id)
        collapsedIds = collapsed
    }
    property string activePanel: "layers"
    signal colorRequested()
    signal focusCanvas()
    property var panelList: [{id:"layers",name:"Camadas"},{id:"inspector",name:"Inspetor"},{id:"color",name:"Cor"},{id:"history",name:"Histórico"}]
    function panelOpen(id) {
        var panels = (studioState.preferences || {}).panels || []
        if (!panels.length) return true
        return panels.some(function(panel) { return panel.id === id && panel.open })
    }
    function execute(name, payload) { return backend.invoke(name, JSON.stringify(payload || {})) }
    function commitTransform(key, value) {
        var selection = studioState.selection || {}
        var payload = {x:selection.x || 0,y:selection.y || 0,width:selection.width || 0,height:selection.height || 0}
        payload[key] = value; execute("transform", payload)
    }
    Connections {
        target: root.backend
        function onStateJsonChanged() {
            if (!root.panelOpen(root.activePanel)) {
                for (var i=0;i<root.panelList.length;i++) if (root.panelOpen(root.panelList[i].id)) { root.activePanel = root.panelList[i].id; break }
            }
        }
    }
    Component {
        id: panelContent
        Rectangle {
            color: root.theme.panel
            ColumnLayout {
                anchors.fill: parent; spacing: 4
                RowLayout {
                    Layout.fillWidth: true; Layout.margins: 8
                    Label { text: "Estúdio"; color: root.theme.secondary; Layout.fillWidth: true }
                    StudioButton {
                        theme: root.theme; backend: root.backend
                        iconKey: root.floating ? "panel.dock" : "panel.float"
                        text: root.floating ? "Acoplar" : "Desacoplar"
                        onClicked: { root.floating = !root.floating; if (!root.floating) root.focusCanvas() }
                    }
                }
                Flow {
                    Layout.fillWidth: true; Layout.leftMargin: 8; Layout.rightMargin: 8
                    spacing: 4
                    Repeater {
                        model: root.panelList
                        StudioButton {
                            required property var modelData
                            visible: root.panelOpen(modelData.id)
                            theme: root.theme; text: modelData.name
                            selectable: true; selected: root.activePanel === modelData.id
                            onClicked: root.activePanel = modelData.id
                        }
                    }
                }
                Rectangle { Layout.fillWidth: true; height: 1; color: root.theme.subtle }
                ScrollView {
                    visible: root.activePanel === "layers" && root.panelOpen("layers")
                    Layout.fillWidth: true; Layout.fillHeight: true
                    clip: true
                    ColumnLayout {
                        width: parent.width; spacing: 8
                        visible: root.activePanel === "layers" && root.panelOpen("layers")
                        Label { Layout.margins: 12; text: "Objetos e camadas"; color: root.theme.secondary }
                        Label { visible: !(root.studioState.layers || []).length; Layout.margins: 12; text: "O documento está vazio."; color: root.theme.foreground }
                        Repeater {
                            model: layerModel
                            RowLayout {
                                id: layerRow; required property var entry; readonly property var modelData: entry
                                visible: root.layerVisible(layerRow.modelData)
                                Layout.fillWidth: true; Layout.leftMargin: 8 + (layerRow.modelData.depth || 0)*12; Layout.rightMargin: 8
                                spacing: 2
                                StudioButton {
                                    objectName: "layerExpand_"+layerRow.modelData.id
                                    theme: root.theme; text: root.collapsedIds.indexOf(layerRow.modelData.id) >= 0 ? "▸" : "▾"
                                    visible: !!layerRow.modelData.hasChildren
                                    Accessible.name: (root.collapsedIds.indexOf(layerRow.modelData.id) >= 0 ? "Expandir " : "Recolher ") + layerRow.modelData.name
                                    onClicked: root.toggleChildren(layerRow.modelData.id)
                                }
                                StudioButton {
                                    Layout.fillWidth: true
                                    theme: root.theme; backend: root.backend
                                    objectName: "layer_"+layerRow.modelData.id
                                    text: layerRow.modelData.name || "Objeto"
                                    iconKey: "object." + (layerRow.modelData.kind || "path")
                                    selectable: true; selected: !!layerRow.modelData.selected
                                    Accessible.description: (layerRow.modelData.label || "") + (layerRow.modelData.locked ? "; bloqueado" : "") + (!layerRow.modelData.visible ? "; oculto" : "")
                                    onClicked: root.execute("selectLayer", {id:layerRow.modelData.id})
                                }
                                StudioButton {
                                    theme: root.theme; backend: root.backend; iconOnly: true
                                    iconKey: layerRow.modelData.visible ? "layer.visible" : "layer.hidden"
                                    text: (layerRow.modelData.visible ? "Ocultar " : "Mostrar ") + layerRow.modelData.name
                                    selectable: true; selected: !!layerRow.modelData.visible
                                    onClicked: root.execute("visibility", {id:layerRow.modelData.id})
                                }
                                StudioButton {
                                    theme: root.theme; backend: root.backend; iconOnly: true
                                    iconKey: layerRow.modelData.locked ? "layer.locked" : "layer.unlocked"
                                    text: (layerRow.modelData.locked ? "Desbloquear " : "Bloquear ") + layerRow.modelData.name
                                    selectable: true; selected: !!layerRow.modelData.locked
                                    onClicked: root.execute("locked", {id:layerRow.modelData.id})
                                }
                            }
                        }
                    }
                }
                ScrollView {
                    visible: root.activePanel === "inspector" && root.panelOpen("inspector")
                    Layout.fillWidth: true; Layout.fillHeight: true; clip: true
                ColumnLayout {
                    width: parent.width-24
                    spacing: 12; x:12; y:12
                    Label { text: "Transformação · unidades do documento"; color: root.theme.secondary; wrapMode: Text.Wrap; Layout.fillWidth: true }
                    Label { visible: !root.studioState.selectionCount; text: "Selecione um objeto para editar."; color: root.theme.foreground }
                    GridLayout {
                        columns: root.width < 320*root.theme.uiScale ? 1 : 2; Layout.fillWidth: true
                        enabled: root.studioState.selectionCount > 0 && (root.studioState.selection || {}).editable !== false
                        Repeater {
                            model: [{key:"x",label:"X"},{key:"y",label:"Y"},{key:"width",label:"Largura"},{key:"height",label:"Altura"}]
                            ColumnLayout {
                                id: numberColumn; required property var modelData
                                Layout.fillWidth: true
                                Label { text: numberColumn.modelData.label; color: root.theme.foreground }
                                NumericField {
                                    objectName: "transform_"+numberColumn.modelData.key
                                    Layout.fillWidth: true
                                    theme: root.theme; fieldName: numberColumn.modelData.label
                                    number: (root.studioState.selection || {})[numberColumn.modelData.key] || 0
                                    minimum: numberColumn.modelData.key === "width" || numberColumn.modelData.key === "height" ? 0.0001 : -1000000
                                    onCommitted: value => root.commitTransform(numberColumn.modelData.key, value)
                                    onCancelled: root.focusCanvas()
                                }
                            }
                        }
                    }
                    NodeInspector {
                        visible: (root.studioState.nodes || []).length > 0
                        Layout.fillWidth: true
                        theme: root.theme; backend: root.backend; studioState: root.studioState
                        onFocusCanvas: root.focusCanvas()
                    }
                    Flow {
                        Layout.fillWidth: true; spacing: 4
                        StudioButton { theme: root.theme; text: "Duplicar"; enabled: root.studioState.selectionCount > 0; onClicked: root.execute("duplicate") }
                        StudioButton { theme: root.theme; text: "Excluir"; enabled: root.studioState.selectionCount > 0; onClicked: root.execute("delete") }
                    }
                }
                }
                ColumnLayout {
                    visible: root.activePanel === "color" && root.panelOpen("color")
                    Layout.fillWidth: true; Layout.margins: 12
                    Label { text: "Preenchimento"; color: root.theme.foreground }
                    Rectangle {
                        Layout.fillWidth: true; height: 48
                        color: (root.studioState.selection || {}).fill || "transparent"
                        border.color: root.theme.controlBorder; border.width: 1; radius: 4
                    }
                    StudioButton {
                        theme: root.theme; text: "Escolher cor…"; enabled: root.studioState.selectionCount > 0
                        onClicked: root.colorRequested()
                    }
                    Label { text: "Hexadecimal (#RRGGBB)"; color: root.theme.secondary }
                    TextField {
                        id: hexColor; property bool dirtyDraft: false; Layout.fillWidth: true
                        text: (root.studioState.selection || {}).fill || "#66D9EF"
                        color: root.theme.foreground
                        Accessible.name: "Cor de preenchimento hexadecimal"
                        validator: RegularExpressionValidator { regularExpression: /^#[0-9a-fA-F]{6}([0-9a-fA-F]{2})?$/ }
                        enabled: root.studioState.selectionCount > 0
                        background: Rectangle { radius: 4; color: root.theme.raised; border.width: hexColor.activeFocus ? 2 : 1; border.color: hexColor.activeFocus ? root.theme.accent : root.theme.controlBorder }
                        onTextEdited: dirtyDraft = true
                        onEditingFinished: { if (dirtyDraft && acceptableInput) root.execute("setFill", {color:text}); dirtyDraft = false }
                    }
                }
                ScrollView {
                    visible: root.activePanel === "history" && root.panelOpen("history")
                    Layout.fillWidth: true; Layout.fillHeight: true; clip: true
                    ColumnLayout {
                        width: parent.width; spacing: 8
                        Label { Layout.margins: 12; text: "Histórico do documento"; color: root.theme.secondary }
                        Repeater {
                            model: root.studioState.history || []
                            Label {
                                required property var modelData
                                Layout.fillWidth: true; Layout.margins: 8
                                text: modelData.label + " · " + modelData.revision
                                color: root.theme.foreground; wrapMode: Text.Wrap
                            }
                        }
                    }
                }
                Item { Layout.fillHeight: true; visible: root.activePanel !== "layers" && root.activePanel !== "history" && root.activePanel !== "inspector" }
            }
        }
    }
    Rectangle { anchors.fill: parent; color: "transparent"; border.width: root.activeFocus ? 2 : 0; border.color: root.theme.accent; z: 10 }
    Loader {
        id: contentLoader
        parent: root.floating ? floatingWindow.contentItem : root
        anchors.fill: parent
        sourceComponent: panelContent
    }
    function focusContent() {
        if (floating) floatingWindow.requestActivate()
        if (contentFocusItem) contentFocusItem.forceActiveFocus(Qt.TabFocusReason)
    }
    Window {
        id: floatingWindow
        title: "Petunia · Painéis"
        width: 360; height: 620; minimumWidth: 300; minimumHeight: 400
        visible: root.floating
        color: root.theme.panel
        transientParent: root.Window.window
        onClosing: close => { close.accepted = false; root.floating = false; root.focusCanvas() }

    }
}
