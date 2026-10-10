import QtQuick
import QtQuick.Controls
import QtTest
import ".."

TestCase {
    id: testCase
    name: "CanvasFrame"
    visible: true; width: 1024; height: 700
    when: windowShown
    StudioTheme { id: tokens }
    QtObject {
        id: fakeBackend
        property string stateJson: "{}"
        property string fixtureFrame: "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABQAAAAUCAYAAACNiR0NAAAANUlEQVR4nGP8////fwYqAiZqGjY0DGRBFxBrWIdV4auGIKIMHPxeHoEGMo7mlFEDRw0kAgAAHXELIe2Okm0AAAAASUVORK5CYII="
        property string frameSource: ""
        function renderFrame(width,height,dpr) { frameSource = fixtureFrame; stateJson = "{\"ready\":true}" }
        function invoke(command,payload) { return true }
    }
    CanvasView { id: canvas; width: 962; height: 659; theme: tokens; backend: fakeBackend; studioState: ({overlays:[]}) }
    function test_large_decoded_frame_is_visible() {
        tryCompare(canvas,"frameReady",true)
        wait(50)
        var snapshot = grabImage(canvas)
        compare(snapshot.pixel(700,500),Qt.rgba(1,1,1,1))
        compare(snapshot.pixel(280,220),Qt.rgba(21/255,127/255,173/255,1))
    }
}
