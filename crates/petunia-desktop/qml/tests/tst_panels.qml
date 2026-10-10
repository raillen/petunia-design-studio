import QtQuick
import QtQuick.Controls
import QtTest
import ".."

TestCase {
    id: testCase
    name: "StudioPanels"
    visible: true; width: 600; height: 700
    when: windowShown
    StudioTheme { id: tokens }
    property var state: ({
        preferences:{panels:[{id:"layers",open:true},{id:"inspector",open:true},{id:"color",open:true},{id:"history",open:true}]},
        layers:[{id:"group",name:"Grupo",group:true,hasChildren:true,visible:true,locked:false,parent:null},{id:"shape",name:"Retângulo",depth:1,parent:"group",visible:true,locked:false}],
        nodes:[],selectionCount:1,selection:{x:12,y:20,width:90,height:45,editable:true,fill:"#66D9EF"},history:[]
    })
    QtObject {
        id: fakeBackend
        property string stateJson: "{}"
        function iconSource(key) { return "" }
        function invoke(command,payload) {
            if (command === "selectLayer") {
                var id = JSON.parse(payload).id
                var next = JSON.parse(JSON.stringify(testCase.state))
                next.layers.forEach(function(layer) { layer.selected = layer.id === id })
                testCase.state = next
            }
            return true
        }
    }
    StudioPanels { id: panels; width: 360; height: 680; theme: tokens; backend: fakeBackend; studioState: testCase.state }
    function cleanup() { panels.floating = false; panels.activePanel = "layers"; panels.collapsedIds = [] }
    function test_group_keyboard_expand_collapse() {
        var expander = findChild(panels,"layerExpand_group")
        verify(expander !== null)
        expander.forceActiveFocus(); wait(10); keyClick(Qt.Key_Space)
        compare(panels.collapsedIds.length,1)
        verify(!panels.layerVisible(state.layers[1]))
        keyClick(Qt.Key_Space)
        compare(panels.collapsedIds.length,0)
        verify(panels.layerVisible(state.layers[1]))
    }
    function test_layer_selection_keeps_keyboard_focus() {
        var original = findChild(panels,"layer_shape")
        verify(original !== null)
        original.forceActiveFocus(); wait(10); keyClick(Qt.Key_Space)
        verify(state.layers[1].selected)
        verify(original.activeFocus)
        verify(findChild(panels,"layer_shape") === original)
        keyClick(Qt.Key_Space)
        verify(original.activeFocus)
    }
    function test_docking_preserves_component() {
        panels.activePanel = "inspector"
        var original = findChild(panels.panelContentItem,"transform_x")
        verify(original !== null)
        panels.floating = true; wait(10)
        // Reparenting keeps the exact existing editor, instead of constructing a new one.
        verify(findChild(panels.panelContentItem,"transform_x") === original)
        panels.floating = false; wait(10)
        verify(findChild(panels.panelContentItem,"transform_x") === original)
    }
}
