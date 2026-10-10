import QtQuick
import QtQuick.Window
import Petunia.Studio

Item {
    id: suite
    Main { id: studio }
    property int stage: 0
    property bool inProgress: false
    property string directory: Qt.application.arguments[Qt.application.arguments.indexOf("--smoke-test") + 1]
    function state() { return JSON.parse(studio.backend.stateJson) }
    function assert(condition, message) { if (!condition) throw new Error(message) }
    function invoke(command, payload) {
        assert(studio.backend.invoke(command, JSON.stringify(payload || {})), command + ": " + state().error)
    }
    function snapshot(name) {
        assert(studio.backend.captureWindow(directory + "/" + name + ".png"), "Screenshot " + name)
    }
    Timer {
        interval: 1000; running: true; repeat: true
        onTriggered: {
            if (suite.inProgress) return
            suite.inProgress = true
            try {
                if (suite.stage === 0) {
                    suite.assert(suite.directory.startsWith("/"), "Explicit test output directory required")
                    suite.invoke("new", {discard:true})
                    suite.invoke("shape", {kind:"rectangle"})
                    suite.invoke("transform", {x:90,y:100,width:420,height:260})
                    suite.invoke("setFill", {color:"#157FAD"})
                    suite.invoke("shape", {kind:"ellipse"})
                    suite.invoke("transform", {x:330,y:190,width:210,height:210})
                    suite.invoke("setFill", {color:"#FFAA66"})
                    var revision = suite.state().revision
                    suite.invoke("undo"); suite.assert(suite.state().canRedo,"Undo enables redo")
                    suite.invoke("redo"); suite.assert(suite.state().revision === revision,"Redo restores the saved revision")
                    suite.invoke("fit")
                    suite.invoke("saveAs", {path:suite.directory+"/drawing.ptnd",overwrite:true})
                    suite.assert(!suite.state().dirty,"Save clears dirty state")
                    suite.invoke("open", {path:suite.directory+"/drawing.ptnd"})
                    suite.assert(suite.state().layers.length===2,"Reopen restores shapes")
                    suite.invoke("exportPng",{path:suite.directory+"/export.png",overwrite:true})
                    suite.invoke("preferences",{appearance:"dark",iconFamily:"phosphor",iconStyle:"outline",uiScale:1})
                    suite.invoke("chooseTool",{tool:"pen"})
                    suite.invoke("pointer",{phase:"down",x:100,y:100})
                    suite.invoke("pointer",{phase:"up",x:100,y:100})
                    suite.assert(suite.state().gestureActive,"Pen remains staged after release")
                    suite.assert(!studio.backend.invoke("new","{}"),"Draft blocks destructive action")
                    suite.invoke("escape"); suite.assert(!suite.state().gestureActive,"Escape cancels staged pen")
                    suite.invoke("chooseTool",{tool:"select"})
                    suite.assert(studio.backend.iconSource("tool.pen").startsWith("data:image/svg+xml;base64,"),"Embedded icon")
                } else if (suite.stage===1 || suite.stage===3 || suite.stage===5) {
                    console.log("CANVAS_READY",studio.canvasView.frameReady,studio.canvasView.frameItem.status,
                        studio.canvasView.width,studio.canvasView.height,studio.canvasView.frameItem.visible)
                    suite.assert(studio.canvasView.frameReady,"Canvas Image is ready")
                    studio.update()
                } else if (suite.stage===2) {
                    suite.assert(studio.backend.frameSource.startsWith("data:image/png;base64,"),"Native canvas frame")
                    suite.snapshot("dark-phosphor-outline")
                    suite.invoke("preferences",{appearance:"light",iconFamily:"tabler",iconStyle:"fill"})
                } else if (suite.stage===4) {
                    suite.snapshot("light-tabler-fill")
                    suite.invoke("preferences",{appearance:"highContrast",uiScale:2})
                } else if (suite.stage===6) {
                    suite.snapshot("contrast-scale-200")
                    suite.invoke("preferences",{appearance:"dark",uiScale:1})
                } else if (suite.stage===8) {
                    console.log("PETUNIA_NATIVE_SMOKE_PASS")
                    Qt.quit()
                }
                suite.stage++
            } catch (error) {
                console.error("PETUNIA_NATIVE_SMOKE_FAIL",String(error))
                Qt.exit(1)
            } finally {
                suite.inProgress = false
            }
        }
    }
}
