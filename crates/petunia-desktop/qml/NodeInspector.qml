pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ColumnLayout {
    id: root
    required property StudioTheme theme
    required property var backend
    required property var studioState
    signal focusCanvas()
    ListModel { id: nodeModel; dynamicRoles: true }
    function syncNodes() {
        if (!nodeModel) return
        var nodes = studioState.nodes || []
        for (var i=0;i<nodes.length;i++) {
            if (i >= nodeModel.count || nodeModel.get(i).entry.id !== nodes[i].id) {
                var found = -1
                for (var j=i+1;j<nodeModel.count;j++) if (nodeModel.get(j).entry.id === nodes[i].id) { found=j; break }
                if (found >= 0) nodeModel.move(found,i,1)
                else nodeModel.insert(i,{entry:nodes[i]})
            }
            nodeModel.setProperty(i,"entry",nodes[i])
        }
        if (nodeModel.count > nodes.length) nodeModel.remove(nodes.length,nodeModel.count-nodes.length)
    }
    onStudioStateChanged: syncNodes()
    Component.onCompleted: syncNodes()
    spacing: 8
    Label { text: "Nós do vetor · coordenadas do documento"; color: root.theme.secondary; wrapMode: Text.Wrap; Layout.fillWidth: true }
    ListView {
        id: nodeList
        Layout.fillWidth: true
        Layout.preferredHeight: Math.min(contentHeight,320*root.theme.uiScale)
        clip: true; spacing: 8
        model: nodeModel
        Accessible.role: Accessible.Tree
        Accessible.name: "Árvore de nós do vetor"
        delegate: Rectangle {
            id: nodeRow
            required property var entry
            readonly property var modelData: entry
            width: nodeList.width; height: content.implicitHeight+16
            radius: 4; color: nodeRow.modelData.selected ? root.theme.selected : root.theme.raised
            border.width: nodeRow.modelData.selected ? 1 : 0; border.color: root.theme.accent
            Accessible.role: Accessible.TreeItem
            Accessible.name: modelData.label + " · " + modelData.kind
            function move(key,value) {
                var node = modelData
                var payload = {object:node.object,contour:node.contour,node:node.node,x:node.x,y:node.y}
                payload[key] = value
                return root.backend.invoke("moveNode",JSON.stringify(payload))
            }
            ColumnLayout {
                id: content; anchors.left: parent.left; anchors.right: parent.right; anchors.margins: 8; y: 8
                StudioButton {
                    Layout.fillWidth: true; theme: root.theme
                    objectName: "node_"+nodeRow.modelData.id
                    text: nodeRow.modelData.label || "Nó " + (nodeRow.modelData.node+1)
                    selectable: true; selected: !!nodeRow.modelData.selected
                    Accessible.description: nodeRow.modelData.kind + "; ID " + nodeRow.modelData.id
                    onClicked: root.backend.invoke("selectNode",JSON.stringify({object:nodeRow.modelData.object,contour:nodeRow.modelData.contour,node:nodeRow.modelData.node}))
                }
                Label { text: "Tipo: " + nodeRow.modelData.kind; color: root.theme.secondary }
                GridLayout {
                    columns: 2; Layout.fillWidth: true
                    Label { text: "X"; color: root.theme.foreground }
                    NumericField { objectName: "nodeX_"+nodeRow.modelData.id; Layout.fillWidth: true; theme: root.theme; number: nodeRow.modelData.x; fieldName: "X de " + nodeRow.modelData.label; onCommitted: value => nodeRow.move("x",value); onCancelled: root.focusCanvas() }
                    Label { text: "Y"; color: root.theme.foreground }
                    NumericField { objectName: "nodeY_"+nodeRow.modelData.id; Layout.fillWidth: true; theme: root.theme; number: nodeRow.modelData.y; fieldName: "Y de " + nodeRow.modelData.label; onCommitted: value => nodeRow.move("y",value); onCancelled: root.focusCanvas() }
                }
            }
        }
        ScrollBar.vertical: ScrollBar {}
    }
}
