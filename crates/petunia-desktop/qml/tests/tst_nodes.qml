import QtQuick
import QtQuick.Controls
import QtTest
import ".."

TestCase {
    id: testCase
    name: "NodeKeyboard"
    visible: true; width: 500; height: 650
    when: windowShown
    StudioTheme { id: tokens }
    property var state: ({nodes:[{id:"stable-node",object:"object",contour:0,node:0,x:12,y:20,kind:"cusp",label:"Nó 1",selected:true}]})
    QtObject {
        id: fakeBackend
        property string stateJson: "{}"
        function iconSource(key) { return "" }
        function invoke(command,json) {
            var payload = JSON.parse(json)
            if (command === "moveNode") {
                // Like the Rust JSON projection, every publication constructs a fresh array/object.
                var next = JSON.parse(JSON.stringify(testCase.state))
                next.nodes[0].x = payload.x; next.nodes[0].y = payload.y
                testCase.state = next
            }
            return true
        }
    }
    NodeInspector { id: inspector; width: 380; theme: tokens; backend: fakeBackend; studioState: testCase.state }
    function test_repeated_nudges_keep_semantic_focus() {
        wait(10)
        var original = findChild(inspector,"nodeX_stable-node")
        verify(original !== null)
        original.forceActiveFocus(); keyClick(Qt.Key_Up)
        compare(state.nodes[0].x,13)
        verify(original.activeFocus)
        verify(findChild(inspector,"nodeX_stable-node") === original)
        keyClick(Qt.Key_Up)
        compare(state.nodes[0].x,14)
        verify(original.activeFocus)
        keyClick(Qt.Key_Return)
        compare(state.nodes[0].x,14)
    }
}
