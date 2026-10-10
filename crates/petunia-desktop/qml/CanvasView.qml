pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Window

FocusScope {
    id: view
    required property StudioTheme theme
    required property var backend
    required property var studioState
    property bool spaceHeld: false
    property bool panning: false
    property point lastPointer: Qt.point(0, 0)
    readonly property alias frameItem: frame
    readonly property bool frameReady: frame.status === Image.Ready
    property bool hovered: pointer.containsMouse
    Accessible.role: Accessible.Canvas
    Accessible.name: "Canvas do documento"
    Accessible.description: "Área de edição. Use a árvore de camadas e o inspetor para editar pelo teclado. Espaço e arraste move a vista."
    activeFocusOnTab: true
    function refresh() { renderTimer.restart() }
    function command(name, args) { return backend.invoke(name, JSON.stringify(args || {})) }
    onWidthChanged: refresh()
    onHeightChanged: refresh()
    Connections { target: view.backend; function onStateJsonChanged() { view.refresh() } }
    Timer {
        id: renderTimer; interval: 16
        onTriggered: if (view.width > 1 && view.height > 1) view.backend.renderFrame(Math.floor(view.width), Math.floor(view.height), view.Screen.devicePixelRatio || 1)
    }
    Component.onCompleted: refresh()
    Rectangle { anchors.fill: parent; color: view.theme.surround }
    Image {
        id: frame; objectName: "canvasFrame"
        anchors.fill: parent
        source: view.backend.frameSource
        cache: false
        asynchronous: false
        fillMode: Image.Stretch
        Accessible.ignored: true
    }
    Canvas {
        id: overlays; objectName: "canvasOverlays"
        visible: (view.studioState.overlays || []).length > 0
        anchors.fill: parent
        onPaint: {
            var painter = getContext("2d")
            painter.clearRect(0, 0, width, height)
            var items = view.studioState.overlays || []
            painter.strokeStyle = String(view.theme.accent)
            painter.fillStyle = String(view.theme.raised)
            painter.lineWidth = 1.5
            items.forEach(function(item) {
                var type = item.type || item.kind
                if (type === "bounds" || type === "marquee" || type === "rectangle" || type === "rect") {
                    painter.setLineDash(type === "marquee" ? [4, 3] : [])
                    if (item.shape === "ellipse") {
                        painter.beginPath(); painter.ellipse(item.x,item.y,item.width,item.height); painter.stroke()
                    } else painter.strokeRect(item.x, item.y, item.width, item.height)
                } else if (type === "ghost") {
                    var points = item.points || []
                    if (points.length) {
                        painter.setLineDash([4,3]); painter.beginPath(); painter.moveTo(points[0].x,points[0].y)
                        for (var point=1;point<points.length;point++) painter.lineTo(points[point].x,points[point].y)
                        painter.stroke()
                    }
                } else if (type === "line" || type === "handleLine") {
                    painter.setLineDash([])
                    painter.beginPath(); painter.moveTo(item.x, item.y); painter.lineTo(item.x2, item.y2); painter.stroke()
                } else if (type === "node" || type === "handle") {
                    var size = item.size || 6
                    painter.setLineDash([])
                    painter.beginPath()
                    if (item.shape === "square" || item.nodeType === "cusp") painter.rect(item.x-size/2, item.y-size/2, size, size)
                    else painter.arc(item.x, item.y, size/2, 0, Math.PI * 2)
                    painter.fillStyle = String(item.selected ? view.theme.accent : view.theme.raised)
                    painter.fill(); painter.stroke()
                    if (item.selected) {
                        painter.beginPath(); painter.arc(item.x,item.y,size/2+4,0,Math.PI*2); painter.stroke()
                    }
                    if (item.shape === "double" || item.nodeType === "symmetric") {
                        painter.beginPath(); painter.arc(item.x, item.y, size/2+3, 0, Math.PI*2); painter.stroke()
                    }
                }
            })
        }
        Connections { target: view.backend; function onStateJsonChanged() { overlays.requestPaint() } }
        Connections { target: view.theme; function onAppearanceChanged() { overlays.requestPaint() } }
    }
    MouseArea {
        id: pointer
        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.LeftButton | Qt.MiddleButton
        cursorShape: view.panning || view.spaceHeld ? Qt.OpenHandCursor : Qt.ArrowCursor
        function send(phase, mouse) {
            view.command("pointer", {phase: phase, x: mouse.x, y: mouse.y, pressure: 1,
                shift: !!(mouse.modifiers & Qt.ShiftModifier), ctrl: !!(mouse.modifiers & Qt.ControlModifier)})
        }
        onPressed: mouse => {
            view.forceActiveFocus()
            view.panning = mouse.button === Qt.MiddleButton || view.spaceHeld
            view.lastPointer = Qt.point(mouse.x, mouse.y)
            if (!view.panning) send("down", mouse)
        }
        onPositionChanged: mouse => {
            if (pressed && view.panning) {
                view.command("pan", {dx: mouse.x - view.lastPointer.x, dy: mouse.y - view.lastPointer.y})
                view.lastPointer = Qt.point(mouse.x, mouse.y)
            } else if (pressed) send("move", mouse)
        }
        onReleased: mouse => { if (!view.panning) send("up", mouse); view.panning = false }
        onCanceled: { view.command("escape"); view.panning = false }
        onDoubleClicked: mouse => { if (!view.spaceHeld) send("double", mouse) }
        onWheel: wheel => {
            if (wheel.modifiers & Qt.ControlModifier) view.command("zoom", {factor: wheel.angleDelta.y > 0 ? 1.2 : 1/1.2, anchorX: wheel.x, anchorY: wheel.y})
            else view.command("pan", {dx: wheel.angleDelta.x / 3, dy: wheel.angleDelta.y / 3})
            wheel.accepted = true
        }
    }
    Rectangle {
        anchors.fill: parent
        color: "transparent"
        border.width: view.activeFocus ? 2 : 0
        border.color: view.theme.accent
    }
    Keys.onPressed: event => {
        if (event.key === Qt.Key_Space) { spaceHeld = true; event.accepted = true }
        else if (event.key === Qt.Key_Left || event.key === Qt.Key_Right || event.key === Qt.Key_Up || event.key === Qt.Key_Down) {
            if (studioState.selectionCount > 0) {
                event.accepted = false
            } else {
                var step = event.modifiers & Qt.ShiftModifier ? 10 : 1
                command("pan", {dx: event.key === Qt.Key_Left ? step*10 : event.key === Qt.Key_Right ? -step*10 : 0,
                    dy: event.key === Qt.Key_Up ? step*10 : event.key === Qt.Key_Down ? -step*10 : 0})
                event.accepted = true
            }
        }
    }
    Keys.onReleased: event => { if (event.key === Qt.Key_Space) { spaceHeld = false; event.accepted = true } }
}
