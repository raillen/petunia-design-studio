import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Shapes
import QtQuick.Dialogs
import "components"
import "panels"
import "dialogs"

ApplicationWindow {
    id: root
    visible: true
    width: 1280
    height: 800
    title: "Petunia Designer — " + bridge.documentName
    color: "#1c1c1d"

    // -------------------------------------------------------------
    // Affinity Designer Color Theme & Styles
    // -------------------------------------------------------------
    readonly property color bgApp: "#1c1c1d"
    readonly property color bgHeader: "#252526"
    readonly property color bgContext: "#282829"
    readonly property color bgToolbar: "#202021"
    readonly property color bgPanel: "#242426"
    readonly property color bgPanelHeader: "#202021"
    readonly property color bgPanelTabActive: "#2b2b2d"
    readonly property color bgPanelTabInactive: "#1f1f20"
    readonly property color bgPanelBody: "#28282a"
    readonly property color bgField: "#181819"
    readonly property color bgCanvasWorkspace: "#242426"
    readonly property color bgArtboard: "#ffffff"

    readonly property color borderDark: "#181819"
    readonly property color borderSubtle: "#333335"
    readonly property color borderMedium: "#424246"

    readonly property color accentCyan: "#00b4d8"
    readonly property color accentBlue: "#1976d2"
    readonly property color accentHover: "#388bfd"

    readonly property color textPrimary: "#f2f2f5"
    readonly property color textSecondary: "#a1a1a6"
    readonly property color textDim: "#6c6c70"

    // Active Tab Indexes / Studio Properties
    property alias topTab: rightStudio.topTab
    property alias midTab: rightStudio.midTab
    property alias bottomTab: rightStudio.bottomTab

    // Canvas pan & view state
    property real panX: 60.0
    property real panY: 40.0
    property string canvasHintText: "Arraste para selecionar. Clique para selecionar prancheta ou objeto. Segure Espaço ou Meio do mouse para Pan."
    property bool spacePressed: false

    // Artboard preset state
    property string currentArtboardPreset: "fhd"
    property real artboardPresetW: 1920
    property real artboardPresetH: 1080
    property bool artboardIsPortrait: false

    property real totalDocWidth: {
        var maxW = bridge.documentWidth
        var arts = bridge.artboards
        for (var i = 0; i < arts.length; i++) {
            maxW = Math.max(maxW, arts[i].x + arts[i].width + 300)
        }
        return Math.max(maxW, 6000)
    }

    property real totalDocHeight: {
        var maxH = bridge.documentHeight
        var arts = bridge.artboards
        for (var i = 0; i < arts.length; i++) {
            maxH = Math.max(maxH, arts[i].y + arts[i].height + 300)
        }
        return Math.max(maxH, 4000)
    }

    function selectArtboardPreset(key, w, h) {
        currentArtboardPreset = key
        if (artboardIsPortrait) {
            artboardPresetW = Math.min(w, h)
            artboardPresetH = Math.max(w, h)
        } else {
            artboardPresetW = Math.max(w, h)
            artboardPresetH = Math.min(w, h)
        }
    }

    function toggleArtboardOrientation() {
        artboardIsPortrait = !artboardIsPortrait
        var tmp = artboardPresetW
        artboardPresetW = artboardPresetH
        artboardPresetH = tmp
    }

    ParallelAnimation {
        id: panAnim
        NumberAnimation { id: panAnimX; target: root; property: "panX"; duration: 220; easing.type: Easing.OutCubic }
        NumberAnimation { id: panAnimY; target: root; property: "panY"; duration: 220; easing.type: Easing.OutCubic }
    }

    Connections {
        target: bridge
        function onViewportFocusRequested(cx, cy, targetZoom) {
            if (targetZoom > 0) bridge.setZoom(targetZoom)
            panAnimX.to = (canvasArea.width / 2) - cx * bridge.zoom
            panAnimY.to = (canvasArea.height / 2) - cy * bridge.zoom
            panAnim.restart()
        }
    }

    // Multi-shape tool cycle
    property string activeShapeType: "rectangle"

    function selectedObject() {
        var objs = bridge.objects
        for (var i = 0; i < objs.length; i++) {
            if (objs[i].id === bridge.selectedId) return objs[i]
        }
        return null
    }

    function drawPathData(ctx, contours) {
        for (var ci = 0; ci < contours.length; ci++) {
            var c = contours[ci]
            if (!c.nodes || !c.nodes.length) continue
            ctx.moveTo(c.nodes[0].x, c.nodes[0].y)
            for (var si = 0; si + 1 < c.nodes.length; si++) {
                var a = c.nodes[si]
                var b = c.nodes[si + 1]
                if (a.outHandle || b.inHandle) {
                    var p1 = a.outHandle ? [a.x + a.outHandle[0], a.y + a.outHandle[1]] : [a.x, a.y]
                    var p2 = b.inHandle ? [b.x + b.inHandle[0], b.y + b.inHandle[1]] : [b.x, b.y]
                    ctx.bezierCurveTo(p1[0], p1[1], p2[0], p2[1], b.x, b.y)
                } else {
                    ctx.lineTo(b.x, b.y)
                }
            }
            if (c.closed) ctx.closePath()
        }
    }

    Component.onCompleted: {
        if (bridge.objects.length === 0) {
            bridge.loadDemoDocument()
        }
        root.panX = Math.max(40, (canvasArea.width - bridge.activeArtboardWidth * bridge.zoom) / 2)
        root.panY = Math.max(40, (canvasArea.height - bridge.activeArtboardHeight * bridge.zoom) / 2)
    }

    // -------------------------------------------------------------
    // Keyboard Shortcuts
    // -------------------------------------------------------------
    Shortcut { sequences: [StandardKey.Undo]; onActivated: bridge.undo() }
    Shortcut { sequences: [StandardKey.Redo, "Ctrl+Shift+Z"]; onActivated: bridge.redo() }
    Shortcut { sequence: "Ctrl+G"; onActivated: bridge.groupSelected() }
    Shortcut { sequence: "Ctrl+Shift+G"; onActivated: bridge.ungroupSelected() }
    Shortcut { sequence: "Ctrl+D"; onActivated: bridge.duplicateSelected() }
    Shortcut { sequence: "Delete"; onActivated: bridge.deleteSelected() }
    Shortcut { sequence: "Backspace"; onActivated: bridge.deleteSelected() }
    Shortcut { sequence: "V"; onActivated: bridge.setTool("select") }
    Shortcut { sequence: "A"; onActivated: bridge.setTool("artboard") }
    Shortcut { sequence: "N"; onActivated: bridge.setTool("node") }
    Shortcut { sequence: "C"; onActivated: bridge.setTool("corner") }
    Shortcut { sequence: "P"; onActivated: bridge.setTool("pen") }
    Shortcut { sequence: "B"; onActivated: bridge.setTool("pencil") }
    Shortcut { sequence: "M"; onActivated: { root.activeShapeType = "rectangle"; bridge.setTool("rectangle") } }
    Shortcut { sequence: "R"; onActivated: { root.activeShapeType = "rectangle"; bridge.setTool("rectangle") } }
    Shortcut { sequence: "O"; onActivated: { root.activeShapeType = "ellipse"; bridge.setTool("ellipse") } }
    Shortcut { sequence: "T"; onActivated: bridge.setTool("text") }
    Shortcut { sequence: "G"; onActivated: bridge.setTool("fill") }
    Shortcut { sequence: "Y"; onActivated: bridge.setTool("transparency") }
    Shortcut { sequence: "I"; onActivated: bridge.setTool("eyedropper") }
    Shortcut { sequence: "H"; onActivated: bridge.setTool("hand") }
    Shortcut { sequence: "Z"; onActivated: bridge.setTool("zoom") }
    Shortcut { sequence: "Return"; onActivated: bridge.penFinish() }
    Shortcut {
        sequence: "Escape"
        onActivated: {
            bridge.penCancel()
            bridge.shapeBuilderCancel()
        }
    }

    // -------------------------------------------------------------
    // Native Desktop MenuBar
    // -------------------------------------------------------------
    menuBar: DesktopMenuBar {
        id: desktopMenuBar
        onRequestNewDocument: newDocDialog.open()
        onRequestDocumentSetup: docSetupDialog.open()
        onRequestAppSettings: appSettingsDialog.open()
        onRequestHelp: helpDialog.open()
        onRequestPlaceImage: placeFileDialog.open()
    }

    // -------------------------------------------------------------
    // Header Bars (Row 1: Main Header, Row 2: Context Toolbar)
    // -------------------------------------------------------------
    header: Column {
        id: headerColumn
        spacing: 0
        width: parent.width

        TopHeaderBar {
            id: topHeaderBar
            onRequestHelp: helpDialog.open()
            onRequestStylesTab: rightStudio.midTab = 3
        }

        ContextToolbar {
            id: contextToolbar
            currentArtboardPreset: root.currentArtboardPreset
            artboardPresetW: root.artboardPresetW
            artboardPresetH: root.artboardPresetH
            artboardIsPortrait: root.artboardIsPortrait
            measureDistance: canvasMouse.measureDistance
            measureAngle: canvasMouse.measureAngle
            onRequestDocumentSetup: docSetupDialog.open()
            onRequestAppSettings: appSettingsDialog.open()
            onRequestPlaceImage: placeFileDialog.open()
            onSelectPreset: function(k, w, h) { root.selectArtboardPreset(k, w, h) }
            onToggleArtboardOrientation: root.toggleArtboardOrientation()
        }
    }

    // -------------------------------------------------------------
    // Global keyboard shortcuts (Space for Pan, A for Artboard, H for Hand, V for Select, Ctrl+0 for Fit)
    // -------------------------------------------------------------
    Item {
        id: globalKeyHandler
        focus: true
        Keys.onPressed: function(event) {
            if (event.key === Qt.Key_Space && !event.isAutoRepeat) {
                root.spacePressed = true
                event.accepted = true
            } else if (event.key === Qt.Key_A && (event.modifiers === Qt.NoModifier)) {
                bridge.setTool("artboard")
                event.accepted = true
            } else if (event.key === Qt.Key_H && (event.modifiers === Qt.NoModifier)) {
                bridge.setTool("hand")
                event.accepted = true
            } else if (event.key === Qt.Key_V && (event.modifiers === Qt.NoModifier)) {
                bridge.setTool("select")
                event.accepted = true
            } else if (event.key === Qt.Key_0 && (event.modifiers & Qt.ControlModifier)) {
                bridge.fitAllArtboards()
                event.accepted = true
            }
        }
        Keys.onReleased: function(event) {
            if (event.key === Qt.Key_Space) {
                root.spacePressed = false
                event.accepted = true
            }
        }
    }

    // -------------------------------------------------------------
    // Main Body Row (Left Toolbar, Center Canvas, Right Sidebar)
    // -------------------------------------------------------------
    RowLayout {
        anchors.fill: parent
        spacing: 0

        // =========================================================
        // LEFT TOOLBAR: Exact Affinity Designer 2 Tool Column
        // =========================================================
        LeftToolbar {
            id: leftToolbar
            editingStroke: rightStudio.editingStroke
            onRequestPathBrushes: rightStudio.midTab = 1
            onRequestColourTab: rightStudio.topTab = 0
            onToolSelected: function(toolId) {
                if (toolId === "rectangle" || toolId === "ellipse") {
                    root.activeShapeType = toolId
                }
            }
        }

        // =========================================================
        // CENTER CANVAS WORKSPACE: Responsive Artboard & Overlays
        // =========================================================
        Rectangle {
            id: canvasArea
            Layout.fillWidth: true
            Layout.fillHeight: true
            color: root.bgCanvasWorkspace
            clip: true

            Item {
                id: docViewport
                x: root.panX
                y: root.panY
                width: root.totalDocWidth
                height: root.totalDocHeight
                scale: bridge.zoom
                transformOrigin: Item.TopLeft

                // -------------------------------------------------------------
                // 1. Multi-Artboard Render Cards
                // -------------------------------------------------------------
                Repeater {
                    model: bridge.artboards
                    delegate: Item {
                        id: artboardItem
                        x: modelData.x
                        y: modelData.y
                        width: modelData.width
                        height: modelData.height
                        z: 0

                        // Title Header Label (above artboard)
                        Rectangle {
                            anchors.bottom: parent.top
                            anchors.left: parent.left
                            anchors.bottomMargin: Math.max(3, 4 / bridge.zoom)
                            height: Math.max(16, 20 / bridge.zoom)
                            width: headerRow.implicitWidth + 10
                            radius: 3
                            color: modelData.selected ? root.accentCyan : "#252528"
                            border.color: modelData.selected ? root.accentHover : "#3e3e42"
                            border.width: 1

                            RowLayout {
                                id: headerRow
                                anchors.centerIn: parent
                                spacing: 4

                                    PIcon {
                                        name: Phosphor.frameCorners
                                        size: Math.max(8, Math.round(10 / bridge.zoom))
                                        color: modelData.selected ? "#ffffff" : root.textSecondary
                                    }

                                    Text {
                                        text: modelData.name
                                        color: modelData.selected ? "#ffffff" : root.textSecondary
                                        font.pixelSize: Math.max(8, Math.round(10 / bridge.zoom))
                                        font.bold: modelData.selected
                                    }

                                Text {
                                    text: Math.round(modelData.width) + " × " + Math.round(modelData.height)
                                    color: modelData.selected ? "#e0f2fe" : root.textDim
                                    font.pixelSize: Math.max(7, Math.round(9 / bridge.zoom))
                                }
                            }

                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onClicked: bridge.select(modelData.id)
                                onDoubleClicked: bridge.focusArtboard(modelData.id)
                            }
                        }

                        // Artboard Canvas Card Background
                        Rectangle {
                            id: cardBg
                            anchors.fill: parent
                            color: modelData.fill || "#ffffff"
                            border.color: modelData.selected ? root.accentCyan : "#38383c"
                            border.width: modelData.selected ? Math.max(1.5, 2.0 / bridge.zoom) : 1
                        }

                        // Resize Handles (visible only when selected and not the fallback document)
                        Item {
                            anchors.fill: parent
                            visible: modelData.selected && !modelData.isDocument

                            // Bottom-Right Corner Handle
                            Rectangle {
                                anchors.right: parent.right
                                anchors.bottom: parent.bottom
                                anchors.rightMargin: -4
                                anchors.bottomMargin: -4
                                width: Math.max(6, 8 / bridge.zoom)
                                height: Math.max(6, 8 / bridge.zoom)
                                color: "#ffffff"
                                border.color: root.accentCyan
                                border.width: 1
                                MouseArea {
                                    anchors.fill: parent
                                    cursorShape: Qt.SizeFDiagCursor
                                    property real startW: 0
                                    property real startH: 0
                                    property real startMouseX: 0
                                    property real startMouseY: 0
                                    onPressed: function(mouse) {
                                        startW = modelData.width
                                        startH = modelData.height
                                        startMouseX = mouse.x
                                        startMouseY = mouse.y
                                    }
                                    onPositionChanged: function(mouse) {
                                        if (!pressed) return
                                        var nw = Math.max(50, startW + (mouse.x - startMouseX))
                                        var nh = Math.max(50, startH + (mouse.y - startMouseY))
                                        bridge.resizeArtboard(modelData.id, nw, nh)
                                    }
                                }
                            }

                            // Right Edge Handle
                            Rectangle {
                                anchors.right: parent.right
                                anchors.verticalCenter: parent.verticalCenter
                                anchors.rightMargin: -4
                                width: Math.max(6, 8 / bridge.zoom)
                                height: Math.max(6, 8 / bridge.zoom)
                                color: "#ffffff"
                                border.color: root.accentCyan
                                border.width: 1
                                MouseArea {
                                    anchors.fill: parent
                                    cursorShape: Qt.SizeHorCursor
                                    property real startW: 0
                                    property real startMouseX: 0
                                    onPressed: function(mouse) {
                                        startW = modelData.width
                                        startMouseX = mouse.x
                                    }
                                    onPositionChanged: function(mouse) {
                                        if (!pressed) return
                                        var nw = Math.max(50, startW + (mouse.x - startMouseX))
                                        bridge.resizeArtboard(modelData.id, nw, modelData.height)
                                    }
                                }
                            }

                            // Bottom Edge Handle
                            Rectangle {
                                anchors.bottom: parent.bottom
                                anchors.horizontalCenter: parent.horizontalCenter
                                anchors.bottomMargin: -4
                                width: Math.max(6, 8 / bridge.zoom)
                                height: Math.max(6, 8 / bridge.zoom)
                                color: "#ffffff"
                                border.color: root.accentCyan
                                border.width: 1
                                MouseArea {
                                    anchors.fill: parent
                                    cursorShape: Qt.SizeVerCursor
                                    property real startH: 0
                                    property real startMouseY: 0
                                    onPressed: function(mouse) {
                                        startH = modelData.height
                                        startMouseY = mouse.y
                                    }
                                    onPositionChanged: function(mouse) {
                                        if (!pressed) return
                                        var nh = Math.max(50, startH + (mouse.y - startMouseY))
                                        bridge.resizeArtboard(modelData.id, modelData.width, nh)
                                    }
                                }
                            }
                        }
                    }
                }

                // -------------------------------------------------------------
                // 2. Vector Document Objects Rendering
                // -------------------------------------------------------------
                Repeater {
                    model: bridge.renderObjects
                    delegate: Item {
                        x: modelData.x
                        y: modelData.y
                        width: modelData.width
                        height: modelData.height
                        visible: modelData.visible
                        z: 1

                        Canvas {
                            anchors.fill: parent
                            property var obj: modelData
                            onObjChanged: requestPaint()
                            onPaint: {
                                var o = obj
                                var ctx = getContext("2d")
                                ctx.clearRect(0, 0, width, height)
                                if (o.type === "group" || o.type === "artboard") return
                                if (o.type === "path" && o.path)
                                    ctx.translate(-o.x, -o.y)

                                ctx.globalAlpha = (o.opacity !== undefined && o.opacity !== null) ? o.opacity : 1.0

                                if (o.gradient && o.gradient.length > 0) {
                                    var g = ctx.createLinearGradient(0, 0, width, 0)
                                    var n = o.gradient.length
                                    for (var i = 0; i < n; i++)
                                        g.addColorStop(n === 1 ? 0 : i / (n - 1), o.gradient[i])
                                    ctx.fillStyle = g
                                } else {
                                    ctx.fillStyle = o.fill
                                }
                                ctx.strokeStyle = o.stroke
                                ctx.lineWidth = o.strokeWidth

                                if (o.type === "text") {
                                    ctx.font = "14px Inter, sans-serif"
                                    ctx.textBaseline = "middle"
                                    var txt = o.text !== undefined ? o.text : (o.name || "Texto")
                                    ctx.fillText(txt, 4, height / 2)
                                } else {
                                    ctx.beginPath()
                                    if (o.type === "ellipse") {
                                        ctx.ellipse(width / 2, height / 2, width / 2, height / 2, 0, 0, Math.PI * 2)
                                    } else if (o.type === "path" && o.path) {
                                        drawPathData(ctx, o.path.contours)
                                    } else if (o.cornerRadius && o.cornerRadius > 0) {
                                        var cr = Math.min(o.cornerRadius, Math.min(width, height) / 2)
                                        ctx.moveTo(cr, 0)
                                        ctx.lineTo(width - cr, 0)
                                        ctx.arcTo(width, 0, width, cr, cr)
                                        ctx.lineTo(width, height - cr)
                                        ctx.arcTo(width, height, width - cr, height, cr)
                                        ctx.lineTo(cr, height)
                                        ctx.arcTo(0, height, 0, height - cr, cr)
                                        ctx.lineTo(0, cr)
                                        ctx.arcTo(0, 0, cr, 0, cr)
                                        ctx.closePath()
                                    } else {
                                        ctx.rect(0, 0, width, height)
                                    }
                                    if (o.fill !== "transparent" && o.fill !== "") {
                                        if (o.type === "path" && o.path)
                                            ctx.fill(o.path.fillRule === "evenodd" ? "evenodd" : "nonzero")
                                        else
                                            ctx.fill()
                                    }
                                    if (o.stroke !== "transparent" && o.stroke !== "" && o.strokeWidth > 0) {
                                        ctx.stroke()
                                    }
                                }
                                ctx.globalAlpha = 1.0
                            }
                        }
                    }
                }

                // -------------------------------------------------------------
                // 3. Interactive Canvas Overlay (Selection box, nodes, pen preview, shape builder)
                // -------------------------------------------------------------
                Canvas {
                    id: overlayCanvas
                    anchors.fill: parent
                    z: 2
                    property var preview: bridge.penPreview
                    property var faces: bridge.shapeFaces
                    property int hoverFace: -1
                    property var selObj: selectedObject()
                    property var node: bridge.selectedNode

                    onPreviewChanged: requestPaint()
                    onFacesChanged: requestPaint()
                    onHoverFaceChanged: requestPaint()
                    onSelObjChanged: requestPaint()
                    onNodeChanged: requestPaint()

                    onPaint: {
                        var ctx = getContext("2d")
                        ctx.clearRect(0, 0, width, height)

                        // 1. Shape Builder Highlight
                        if (bridge.shapeBuilderActive) {
                            for (var fi = 0; fi < faces.length; fi++) {
                                var f = faces[fi]
                                var pts = f.points
                                if (!pts || !pts.length) continue
                                ctx.beginPath()
                                ctx.moveTo(pts[0][0], pts[0][1])
                                for (var pi = 1; pi < pts.length; pi++)
                                    ctx.lineTo(pts[pi][0], pts[pi][1])
                                ctx.closePath()
                                if (f.selected) {
                                    ctx.fillStyle = "rgba(0, 180, 216, 0.45)"
                                    ctx.fill()
                                    ctx.strokeStyle = root.accentCyan
                                    ctx.lineWidth = 1.5
                                    ctx.stroke()
                                } else if (fi === hoverFace) {
                                    ctx.fillStyle = "rgba(0, 180, 216, 0.20)"
                                    ctx.fill()
                                    ctx.strokeStyle = root.accentCyan
                                    ctx.lineWidth = 1
                                    ctx.stroke()
                                }
                            }
                        }

                        // 2. Pen Tool Construction Path
                        if (bridge.penActive && preview) {
                            ctx.strokeStyle = "#ffd166"
                            ctx.lineWidth = 1.5
                            ctx.setLineDash([5, 4])
                            ctx.beginPath()
                            var lastPenNode = null
                            for (var pci = 0; pci < preview.length; pci++) {
                                var pc = preview[pci]
                                if (!pc.nodes || !pc.nodes.length) continue
                                ctx.moveTo(pc.nodes[0].x, pc.nodes[0].y)
                                for (var sni = 0; sni + 1 < pc.nodes.length; sni++) {
                                    var na = pc.nodes[sni]
                                    var nb = pc.nodes[sni + 1]
                                    if (na.outHandle || nb.inHandle) {
                                        var p1 = na.outHandle ? [na.x + na.outHandle[0], na.y + na.outHandle[1]] : [na.x, na.y]
                                        var p2 = nb.inHandle ? [nb.x + nb.inHandle[0], nb.y + nb.inHandle[1]] : [nb.x, nb.y]
                                        ctx.bezierCurveTo(p1[0], p1[1], p2[0], p2[1], nb.x, nb.y)
                                    } else {
                                        ctx.lineTo(nb.x, nb.y)
                                    }
                                }
                                if (pc.nodes.length > 0) {
                                    lastPenNode = pc.nodes[pc.nodes.length - 1]
                                }
                                if (pc.closed) ctx.closePath()
                            }
                            ctx.stroke()
                            ctx.setLineDash([])

                            // Elastic Rubber-band Guide Line from last node to current cursor
                            if (lastPenNode && !canvasMouse.penIsDraggingHandle && canvasMouse.containsMouse && (canvasMouse.currentDocX !== 0 || canvasMouse.currentDocY !== 0)) {
                                ctx.strokeStyle = "rgba(0, 180, 216, 0.75)"
                                ctx.lineWidth = 1.2
                                ctx.setLineDash([3, 3])
                                ctx.beginPath()
                                ctx.moveTo(lastPenNode.x, lastPenNode.y)
                                ctx.lineTo(canvasMouse.currentDocX, canvasMouse.currentDocY)
                                ctx.stroke()
                                ctx.setLineDash([])
                            }

                            // Render node points and tangent handles for all pen preview nodes
                            for (var pci2 = 0; pci2 < preview.length; pci2++) {
                                var pc2 = preview[pci2]
                                if (!pc2.nodes) continue
                                for (var pni = 0; pni < pc2.nodes.length; pni++) {
                                    var pn = pc2.nodes[pni]
                                    // Tangent handles
                                    if (pn.inHandle) {
                                        var phix = pn.x + pn.inHandle[0]
                                        var phiy = pn.y + pn.inHandle[1]
                                        ctx.strokeStyle = root.accentCyan
                                        ctx.lineWidth = 1
                                        ctx.beginPath(); ctx.moveTo(pn.x, pn.y); ctx.lineTo(phix, phiy); ctx.stroke()
                                        ctx.fillStyle = "#ffd166"; ctx.strokeStyle = "#1e1e20"
                                        ctx.beginPath(); ctx.arc(phix, phiy, 3.5, 0, Math.PI * 2); ctx.fill(); ctx.stroke()
                                    }
                                    if (pn.outHandle) {
                                        var phox = pn.x + pn.outHandle[0]
                                        var phoy = pn.y + pn.outHandle[1]
                                        ctx.strokeStyle = root.accentCyan
                                        ctx.lineWidth = 1
                                        ctx.beginPath(); ctx.moveTo(pn.x, pn.y); ctx.lineTo(phox, phoy); ctx.stroke()
                                        ctx.fillStyle = "#ffd166"; ctx.strokeStyle = "#1e1e20"
                                        ctx.beginPath(); ctx.arc(phox, phoy, 3.5, 0, Math.PI * 2); ctx.fill(); ctx.stroke()
                                    }
                                    // Node grip
                                    ctx.fillStyle = "#ffffff"
                                    ctx.strokeStyle = root.accentCyan
                                    ctx.lineWidth = 1.5
                                    ctx.beginPath()
                                    ctx.arc(pn.x, pn.y, 4, 0, Math.PI * 2)
                                    ctx.fill()
                                    ctx.stroke()
                                }
                            }
                        }

                        // 3. Selection Bounding Box & Handles / Crop Box / Artboard Box
                        var o = selObj
                        if (o && o.type !== "group" && bridge.selectedIds.length > 0 && !(o.type === "artboard" && bridge.tool !== "artboard" && bridge.tool !== "select")) {
                            if (bridge.tool === "crop") {
                                // Vector Crop Box (Rule-of-thirds grid + heavy brackets)
                                ctx.strokeStyle = "#f59e0b"
                                ctx.lineWidth = 1.5
                                ctx.strokeRect(o.x, o.y, o.width, o.height)

                                // Rule of thirds grid lines
                                ctx.strokeStyle = "rgba(245, 158, 11, 0.4)"
                                ctx.lineWidth = 1
                                ctx.setLineDash([3, 3])
                                ctx.beginPath()
                                ctx.moveTo(o.x + o.width / 3, o.y); ctx.lineTo(o.x + o.width / 3, o.y + o.height)
                                ctx.moveTo(o.x + 2 * o.width / 3, o.y); ctx.lineTo(o.x + 2 * o.width / 3, o.y + o.height)
                                ctx.moveTo(o.x, o.y + o.height / 3); ctx.lineTo(o.x + o.width, o.y + o.height / 3)
                                ctx.moveTo(o.x, o.y + 2 * o.height / 3); ctx.lineTo(o.x + o.width, o.y + 2 * o.height / 3)
                                ctx.stroke()
                                ctx.setLineDash([])

                                // Heavy crop corner brackets
                                var bl = 12
                                ctx.strokeStyle = "#ffffff"
                                ctx.lineWidth = 3
                                ctx.beginPath()
                                // NW
                                ctx.moveTo(o.x, o.y + bl); ctx.lineTo(o.x, o.y); ctx.lineTo(o.x + bl, o.y)
                                // NE
                                ctx.moveTo(o.x + o.width - bl, o.y); ctx.lineTo(o.x + o.width, o.y); ctx.lineTo(o.x + o.width, o.y + bl)
                                // SW
                                ctx.moveTo(o.x, o.y + o.height - bl); ctx.lineTo(o.x, o.y + o.height); ctx.lineTo(o.x + bl, o.y + o.height)
                                // SE
                                ctx.moveTo(o.x + o.width - bl, o.y + o.height); ctx.lineTo(o.x + o.width, o.y + o.height); ctx.lineTo(o.x + o.width, o.y + o.height - bl)
                                ctx.stroke()

                                // T-bars on midpoints
                                ctx.beginPath()
                                ctx.moveTo(o.x + o.width / 2 - bl / 2, o.y); ctx.lineTo(o.x + o.width / 2 + bl / 2, o.y)
                                ctx.moveTo(o.x + o.width / 2 - bl / 2, o.y + o.height); ctx.lineTo(o.x + o.width / 2 + bl / 2, o.y + o.height)
                                ctx.moveTo(o.x, o.y + o.height / 2 - bl / 2); ctx.lineTo(o.x, o.y + o.height / 2 + bl / 2)
                                ctx.moveTo(o.x + o.width, o.y + o.height / 2 - bl / 2); ctx.lineTo(o.x + o.width, o.y + o.height / 2 + bl / 2)
                                ctx.stroke()
                            } else if (bridge.tool === "corner") {
                                // Corner Tool Calipers & Handles
                                var crVal = bridge.selectedCornerRadius || 0
                                var cRadius = Math.min(crVal, Math.min(o.width, o.height) / 2)

                                ctx.strokeStyle = "#f59e0b"
                                ctx.lineWidth = 1.2
                                ctx.strokeRect(o.x, o.y, o.width, o.height)

                                var cPoints = [
                                    [o.x + cRadius, o.y + cRadius],
                                    [o.x + o.width - cRadius, o.y + cRadius],
                                    [o.x + cRadius, o.y + o.height - cRadius],
                                    [o.x + o.width - cRadius, o.y + o.height - cRadius]
                                ]
                                for (var cpi = 0; cpi < cPoints.length; cpi++) {
                                    ctx.fillStyle = "#ffd166"
                                    ctx.strokeStyle = "#181819"
                                    ctx.lineWidth = 1.5
                                    ctx.beginPath()
                                    ctx.arc(cPoints[cpi][0], cPoints[cpi][1], 4.5, 0, Math.PI * 2)
                                    ctx.fill()
                                    ctx.stroke()
                                }

                                var cBadgeW = 84
                                var cBadgeH = 20
                                var cbx = o.x + o.width / 2 - cBadgeW / 2
                                var cby = o.y - 28
                                ctx.fillStyle = "#1e1e20"
                                ctx.fillRect(cbx, cby, cBadgeW, cBadgeH)
                                ctx.strokeStyle = "#f59e0b"
                                ctx.lineWidth = 1
                                ctx.strokeRect(cbx, cby, cBadgeW, cBadgeH)
                                ctx.fillStyle = "#ffffff"
                                ctx.font = "10px Inter, sans-serif"
                                ctx.textAlign = "center"
                                ctx.textBaseline = "middle"
                                ctx.fillText("Raio: " + Math.round(crVal) + " px", cbx + cBadgeW / 2, cby + cBadgeH / 2)
                            } else {
                                ctx.strokeStyle = root.accentCyan
                                ctx.lineWidth = 1.2
                                ctx.strokeRect(o.x, o.y, o.width, o.height)
                                var hs = 6
                                ctx.fillStyle = "#ffffff"
                                var corners = [
                                    [o.x, o.y], [o.x + o.width, o.y],
                                    [o.x, o.y + o.height], [o.x + o.width, o.y + o.height],
                                    [o.x + o.width / 2, o.y], [o.x + o.width / 2, o.y + o.height],
                                    [o.x, o.y + o.height / 2], [o.x + o.width, o.y + o.height / 2]
                                ]
                                for (var k = 0; k < corners.length; k++) {
                                    ctx.fillRect(corners[k][0] - hs / 2, corners[k][1] - hs / 2, hs, hs)
                                    ctx.strokeRect(corners[k][0] - hs / 2, corners[k][1] - hs / 2, hs, hs)
                                }
                            }
                        }

                        // 4. Node Tool Nodes & Handles
                        if (o && o.type === "path" && o.path && bridge.tool === "node") {
                            for (var ci = 0; ci < o.path.contours.length; ci++) {
                                var contour = o.path.contours[ci]
                                for (var ni = 0; ni < contour.nodes.length; ni++) {
                                    var nd = contour.nodes[ni]
                                    var isSel = node && node.objId === o.id && node.contourIndex === ci && node.nodeIndex === ni
                                    if (isSel) {
                                        if (nd.inHandle) {
                                            var ihx = nd.x + nd.inHandle[0]
                                            var ihy = nd.y + nd.inHandle[1]
                                            ctx.strokeStyle = root.accentCyan
                                            ctx.lineWidth = 1
                                            ctx.beginPath(); ctx.moveTo(nd.x, nd.y); ctx.lineTo(ihx, ihy); ctx.stroke()
                                            ctx.fillStyle = "#ffffff"; ctx.strokeStyle = root.accentCyan
                                            ctx.beginPath(); ctx.arc(ihx, ihy, 3.5, 0, Math.PI * 2); ctx.fill(); ctx.stroke()
                                        }
                                        if (nd.outHandle) {
                                            var ohx = nd.x + nd.outHandle[0]
                                            var ohy = nd.y + nd.outHandle[1]
                                            ctx.strokeStyle = root.accentCyan
                                            ctx.lineWidth = 1
                                            ctx.beginPath(); ctx.moveTo(nd.x, nd.y); ctx.lineTo(ohx, ohy); ctx.stroke()
                                            ctx.fillStyle = "#ffffff"; ctx.strokeStyle = root.accentCyan
                                            ctx.beginPath(); ctx.arc(ohx, ohy, 3.5, 0, Math.PI * 2); ctx.fill(); ctx.stroke()
                                        }
                                    }
                                    ctx.fillStyle = isSel ? root.accentCyan : "#ffffff"
                                    ctx.strokeStyle = "#1e1e1e"
                                    ctx.lineWidth = 1
                                    var sz = isSel ? 7 : 5
                                    ctx.fillRect(nd.x - sz / 2, nd.y - sz / 2, sz, sz)
                                    ctx.strokeRect(nd.x - sz / 2, nd.y - sz / 2, sz, sz)
                                }
                            }
                        }

                        // 5. Artboard Tool Live Drag Preview
                        if (canvasMouse.isCreatingArtboard) {
                            var ax = Math.min(canvasMouse.artboardStartX, canvasMouse.artboardCurrentX)
                            var ay = Math.min(canvasMouse.artboardStartY, canvasMouse.artboardCurrentY)
                            var aw = Math.abs(canvasMouse.artboardCurrentX - canvasMouse.artboardStartX)
                            var ah = Math.abs(canvasMouse.artboardCurrentY - canvasMouse.artboardStartY)

                            ctx.fillStyle = "rgba(0, 180, 216, 0.12)"
                            ctx.fillRect(ax, ay, aw, ah)

                            ctx.strokeStyle = root.accentCyan
                            ctx.lineWidth = 1.5
                            ctx.setLineDash([5, 4])
                            ctx.strokeRect(ax, ay, aw, ah)
                            ctx.setLineDash([])

                            // Floating dimension badge
                            var badgeW = 96
                            var badgeH = 22
                            var bx = ax + aw / 2 - badgeW / 2
                            var by = ay + ah + 8
                            ctx.fillStyle = "#1e1e20"
                            ctx.fillRect(bx, by, badgeW, badgeH)
                            ctx.strokeStyle = root.accentCyan
                            ctx.lineWidth = 1
                            ctx.strokeRect(bx, by, badgeW, badgeH)
                            ctx.fillStyle = "#ffffff"
                            ctx.font = "11px Inter, sans-serif"
                            ctx.textAlign = "center"
                            ctx.textBaseline = "middle"
                            ctx.fillText(Math.round(aw) + " × " + Math.round(ah) + " px", bx + badgeW / 2, by + badgeH / 2)
                        }

                        // 6. Shape Creation Live Drag Preview (Rectangle / Ellipse)
                        if (canvasMouse.isDrawingShape && (bridge.tool === "rectangle" || bridge.tool === "ellipse")) {
                            var sx = Math.min(canvasMouse.shapeStartX, canvasMouse.shapeCurrentX)
                            var sy = Math.min(canvasMouse.shapeStartY, canvasMouse.shapeCurrentY)
                            var sw = Math.abs(canvasMouse.shapeCurrentX - canvasMouse.shapeStartX)
                            var sh = Math.abs(canvasMouse.shapeCurrentY - canvasMouse.shapeStartY)

                            ctx.fillStyle = "rgba(0, 180, 216, 0.15)"
                            ctx.strokeStyle = root.accentCyan
                            ctx.lineWidth = 1.5
                            ctx.setLineDash([5, 4])
                            ctx.beginPath()
                            if (bridge.tool === "ellipse") {
                                ctx.ellipse(sx + sw / 2, sy + sh / 2, sw / 2, sh / 2, 0, 0, Math.PI * 2)
                            } else {
                                ctx.rect(sx, sy, sw, sh)
                            }
                            ctx.fill()
                            ctx.stroke()
                            ctx.setLineDash([])

                            var sBadgeW = 86
                            var sBadgeH = 20
                            var sbx = sx + sw / 2 - sBadgeW / 2
                            var sby = sy + sh + 8
                            ctx.fillStyle = "#1e1e20"
                            ctx.fillRect(sbx, sby, sBadgeW, sBadgeH)
                            ctx.strokeStyle = root.accentCyan
                            ctx.lineWidth = 1
                            ctx.strokeRect(sbx, sby, sBadgeW, sBadgeH)
                            ctx.fillStyle = "#ffffff"
                            ctx.font = "10px Inter, sans-serif"
                            ctx.textAlign = "center"
                            ctx.textBaseline = "middle"
                            ctx.fillText(Math.round(sw) + " × " + Math.round(sh) + " px", sbx + sBadgeW / 2, sby + sBadgeH / 2)
                        }

                        // 7. Pencil / Vector Brush Live Stroke Preview
                        if (canvasMouse.isDrawingStroke && canvasMouse.strokePoints.length > 1) {
                            var spts = canvasMouse.strokePoints
                            ctx.strokeStyle = bridge.currentStroke || "#00b4d8"
                            ctx.lineWidth = bridge.tool === "brush" ? Math.max(3, bridge.currentStrokeWidth * 1.5) : bridge.currentStrokeWidth
                            ctx.lineCap = "round"
                            ctx.lineJoin = "round"
                            ctx.beginPath()
                            ctx.moveTo(spts[0][0], spts[0][1])
                            for (var spi = 1; spi < spts.length; spi++) {
                                ctx.lineTo(spts[spi][0], spts[spi][1])
                            }
                            ctx.stroke()
                        }

                        // 8. Measure Tool Live Ruler Preview
                        if (canvasMouse.isMeasuring) {
                            var mx1 = canvasMouse.measureStartX
                            var my1 = canvasMouse.measureStartY
                            var mx2 = canvasMouse.measureCurrentX
                            var my2 = canvasMouse.measureCurrentY
                            var mdx = mx2 - mx1
                            var mdy = my2 - my1
                            var mdist = Math.sqrt(mdx * mdx + mdy * mdy)
                            var mang = Math.atan2(mdy, mdx) * 180 / Math.PI

                            ctx.strokeStyle = "#fef08a"
                            ctx.lineWidth = 1.5
                            ctx.beginPath()
                            ctx.moveTo(mx1, my1)
                            ctx.lineTo(mx2, my2)
                            ctx.stroke()

                            var tickL = 6
                            var pX = -mdy / (mdist || 1) * tickL
                            var pY = mdx / (mdist || 1) * tickL
                            ctx.beginPath()
                            ctx.moveTo(mx1 - pX, my1 - pY); ctx.lineTo(mx1 + pX, my1 + pY)
                            ctx.moveTo(mx2 - pX, my2 - pY); ctx.lineTo(mx2 + pX, my2 + pY)
                            ctx.stroke()

                            var mText = Math.round(mdist) + " px (" + Math.round(mang) + "°)"
                            var mBW = 110
                            var mBH = 22
                            var mbx2 = (mx1 + mx2) / 2 - mBW / 2
                            var mby2 = (my1 + my2) / 2 - 28
                            ctx.fillStyle = "#1e1e20"
                            ctx.fillRect(mbx2, mby2, mBW, mBH)
                            ctx.strokeStyle = "#fef08a"
                            ctx.lineWidth = 1
                            ctx.strokeRect(mbx2, mby2, mBW, mBH)
                            ctx.fillStyle = "#ffffff"
                            ctx.font = "11px Inter, sans-serif"
                            ctx.textAlign = "center"
                            ctx.textBaseline = "middle"
                            ctx.fillText(mText, mbx2 + mBW / 2, mby2 + mBH / 2)
                        }

                        // 9. Fill Tool Gradient Vector Preview
                        if (bridge.tool === "fill" && o && bridge.selectedIds.length > 0) {
                            var gx1 = canvasMouse.isDraggingGradient ? canvasMouse.gradientStartX : o.x
                            var gy1 = canvasMouse.isDraggingGradient ? canvasMouse.gradientStartY : (o.y + o.height / 2)
                            var gx2 = canvasMouse.isDraggingGradient ? canvasMouse.gradientCurrentX : (o.x + o.width)
                            var gy2 = canvasMouse.isDraggingGradient ? canvasMouse.gradientCurrentY : (o.y + o.height / 2)

                            ctx.strokeStyle = "#38bdf8"
                            ctx.lineWidth = 2
                            ctx.beginPath()
                            ctx.moveTo(gx1, gy1)
                            ctx.lineTo(gx2, gy2)
                            ctx.stroke()

                            ctx.fillStyle = bridge.currentFill
                            ctx.strokeStyle = "#ffffff"
                            ctx.lineWidth = 2
                            ctx.beginPath()
                            ctx.arc(gx1, gy1, 6, 0, Math.PI * 2)
                            ctx.fill()
                            ctx.stroke()

                            ctx.fillStyle = (o.gradient && o.gradient.length > 1) ? o.gradient[o.gradient.length - 1] : "#00b4d8"
                            ctx.beginPath()
                            ctx.arc(gx2, gy2, 6, 0, Math.PI * 2)
                            ctx.fill()
                            ctx.stroke()
                        }
                    }
                }
            }

            // Canvas Interaction MouseArea
            MouseArea {
                id: canvasMouse
                anchors.fill: parent
                acceptedButtons: Qt.LeftButton | Qt.MiddleButton
                hoverEnabled: true
                property real lastX: 0
                property real lastY: 0
                property bool dragActive: false
                property var dragNode: null
                property bool isPanning: false

                property bool isCreatingArtboard: false
                property real artboardStartX: 0
                property real artboardStartY: 0
                property real artboardCurrentX: 0
                property real artboardCurrentY: 0

                property bool isDrawingShape: false
                property real shapeStartX: 0
                property real shapeStartY: 0
                property real shapeCurrentX: 0
                property real shapeCurrentY: 0

                property bool isDrawingStroke: false
                property var strokePoints: []

                property bool isMeasuring: false
                property real measureStartX: 0
                property real measureStartY: 0
                property real measureCurrentX: 0
                property real measureCurrentY: 0
                property real measureDistance: 0
                property real measureAngle: 0

                property bool isDraggingGradient: false
                property real gradientStartX: 0
                property real gradientStartY: 0
                property real gradientCurrentX: 0
                property real gradientCurrentY: 0

                property bool penIsDraggingHandle: false
                property real currentDocX: 0
                property real currentDocY: 0

                property string hoveredHandle: ""
                property string activeHandle: ""
                property real origX: 0
                property real origY: 0
                property real origW: 0
                property real origH: 0

                property bool isDraggingCorner: false
                property real cornerStartRadius: 0

                property bool isDraggingTransparency: false
                property real transparencyStartVal: 1.0

                function hitTestHandles(px, py) {
                    if ((bridge.tool !== "select" && bridge.tool !== "crop" && bridge.tool !== "artboard") || bridge.selectedIds.length === 0) return ""
                    var o = selObj
                    if (!o || o.type === "group") return ""
                    if (o.type === "artboard" && bridge.tool !== "artboard" && bridge.tool !== "select") return ""
                    var hs = 8 / bridge.zoom
                    var handles = [
                        { id: "nw", x: o.x, y: o.y },
                        { id: "ne", x: o.x + o.width, y: o.y },
                        { id: "sw", x: o.x, y: o.y + o.height },
                        { id: "se", x: o.x + o.width, y: o.y + o.height },
                        { id: "n",  x: o.x + o.width / 2, y: o.y },
                        { id: "s",  x: o.x + o.width / 2, y: o.y + o.height },
                        { id: "w",  x: o.x, y: o.y + o.height / 2 },
                        { id: "e",  x: o.x + o.width, y: o.y + o.height / 2 }
                    ]
                    for (var i = 0; i < handles.length; i++) {
                        var h = handles[i]
                        if (Math.abs(px - h.x) <= hs && Math.abs(py - h.y) <= hs) {
                            return h.id
                        }
                    }
                    return ""
                }

                cursorShape: {
                    if (isPanning || root.spacePressed || bridge.tool === "hand") return pressed ? Qt.ClosedHandCursor : Qt.OpenHandCursor
                    if (hoveredHandle === "nw" || hoveredHandle === "se") return Qt.SizeFDiagCursor
                    if (hoveredHandle === "ne" || hoveredHandle === "sw") return Qt.SizeBDiagCursor
                    if (hoveredHandle === "n" || hoveredHandle === "s") return Qt.SizeVerCursor
                    if (hoveredHandle === "w" || hoveredHandle === "e") return Qt.SizeHorCursor
                    if (bridge.tool === "artboard" || bridge.tool === "rectangle" || bridge.tool === "ellipse" || bridge.tool === "measure" || bridge.tool === "pixel_select") return Qt.CrossCursor
                    if (bridge.tool === "zoom") return Qt.PointingHandCursor
                    if (bridge.tool === "node" || bridge.tool === "pen") return Qt.CrossCursor
                    if (bridge.tool === "pencil" || bridge.tool === "brush" || bridge.tool === "pixel_brush" || bridge.tool === "pixel_eraser") return Qt.CrossCursor
                    if (bridge.tool === "corner" || bridge.tool === "crop") return Qt.CrossCursor
                    if (bridge.tool === "eyedropper") return Qt.CrossCursor
                    if (bridge.tool === "text") return Qt.IBeamCursor
                    return Qt.ArrowCursor
                }

                function docPos(mouse) {
                    return {
                        x: (mouse.x - root.panX) / bridge.zoom,
                        y: (mouse.y - root.panY) / bridge.zoom
                    }
                }

                onPressed: function(mouse) {
                    if (mouse.button === Qt.MiddleButton || bridge.tool === "hand" || root.spacePressed) {
                        isPanning = true
                        lastX = mouse.x
                        lastY = mouse.y
                        return
                    }

                    var p = docPos(mouse)
                    lastX = p.x
                    lastY = p.y

                    if (bridge.tool === "corner" && bridge.selectedId) {
                        isDraggingCorner = true
                        cornerStartRadius = bridge.selectedCornerRadius
                        return
                    }

                    if (bridge.tool === "transparency" && bridge.selectedId) {
                        isDraggingTransparency = true
                        transparencyStartVal = bridge.selectedOpacity
                        return
                    }

                    if ((bridge.tool === "select" || bridge.tool === "crop" || bridge.tool === "artboard") && bridge.selectedIds.length > 0 && selObj) {
                        var hitH = hitTestHandles(p.x, p.y)
                        if (hitH !== "") {
                            activeHandle = hitH
                            origX = selObj.x
                            origY = selObj.y
                            origW = selObj.width
                            origH = selObj.height
                            dragActive = true
                            return
                        }
                    }

                    if (bridge.tool === "artboard") {
                        // Check if an existing artboard was clicked to select/move
                        var hitArtboardId = ""
                        for (var a = 0; a < bridge.artboards.length; a++) {
                            var ab = bridge.artboards[a]
                            if (p.x >= ab.x && p.x <= ab.x + ab.width && p.y >= ab.y && p.y <= ab.y + ab.height) {
                                hitArtboardId = ab.id
                            }
                        }
                        if (hitArtboardId !== "" && hitArtboardId !== "__doc__") {
                            bridge.select(hitArtboardId)
                            activeHandle = ""
                            origX = selObj ? selObj.x : 0
                            origY = selObj ? selObj.y : 0
                            dragActive = true
                            overlayCanvas.requestPaint()
                            return
                        }

                        isCreatingArtboard = true
                        artboardStartX = p.x
                        artboardStartY = p.y
                        artboardCurrentX = p.x
                        artboardCurrentY = p.y
                        overlayCanvas.requestPaint()
                        return
                    }

                    if (bridge.tool === "rectangle" || bridge.tool === "ellipse" || bridge.tool === "text") {
                        isDrawingShape = true
                        shapeStartX = p.x
                        shapeStartY = p.y
                        shapeCurrentX = p.x
                        shapeCurrentY = p.y
                        overlayCanvas.requestPaint()
                        return
                    }

                    if (bridge.tool === "pencil" || bridge.tool === "brush" || bridge.tool === "pixel_brush" || bridge.tool === "pixel_eraser") {
                        isDrawingStroke = true
                        strokePoints = [[p.x, p.y]]
                        overlayCanvas.requestPaint()
                        return
                    }

                    if (bridge.tool === "measure") {
                        isMeasuring = true
                        measureStartX = p.x
                        measureStartY = p.y
                        measureCurrentX = p.x
                        measureCurrentY = p.y
                        measureDistance = 0
                        measureAngle = 0
                        overlayCanvas.requestPaint()
                        return
                    }

                    if (bridge.tool === "fill") {
                        if (bridge.selectedId) {
                            isDraggingGradient = true
                            gradientStartX = p.x
                            gradientStartY = p.y
                            gradientCurrentX = p.x
                            gradientCurrentY = p.y
                            overlayCanvas.requestPaint()
                        }
                        return
                    }

                    if (bridge.tool === "eyedropper") {
                        bridge.sampleColorAt(p.x, p.y, (mouse.modifiers & Qt.AltModifier))
                        return
                    }

                    if (bridge.shapeBuilderActive) {
                        var faceIdx = bridge.shapeBuilderHit(p.x, p.y)
                        if (faceIdx >= 0) bridge.shapeBuilderToggle(faceIdx)
                        return
                    }

                    if (bridge.tool === "pen") {
                        bridge.penClick(p.x, p.y)
                        penIsDraggingHandle = true
                        return
                    }

                    if (bridge.tool === "node") {
                        var hit = bridge.nodeAt(p.x, p.y)
                        dragNode = hit && hit.objId ? hit : null
                        dragActive = dragNode !== null
                        overlayCanvas.requestPaint()
                        return
                    }

                    if (bridge.tool === "select") {
                        var hitId = bridge.objectAt(p.x, p.y)
                        if (mouse.modifiers & Qt.ShiftModifier) {
                            bridge.toggleSelect(hitId)
                        } else {
                            bridge.select(hitId)
                        }
                        if (hitId !== "") dragActive = true
                    } else if (bridge.tool === "zoom") {
                        if (mouse.modifiers & Qt.AltModifier) bridge.zoomOut()
                        else bridge.zoomIn()
                    }
                }

                onPositionChanged: function(mouse) {
                    if (isPanning) {
                        root.panX += (mouse.x - lastX)
                        root.panY += (mouse.y - lastY)
                        lastX = mouse.x
                        lastY = mouse.y
                        return
                    }

                    var p = docPos(mouse)
                    currentDocX = p.x
                    currentDocY = p.y

                    if (bridge.penActive || bridge.tool === "pen" || bridge.tool === "node" || bridge.tool === "corner" || bridge.tool === "crop") {
                        overlayCanvas.requestPaint()
                    }

                    if (!pressed && (bridge.tool === "select" || bridge.tool === "crop" || bridge.tool === "artboard")) {
                        hoveredHandle = hitTestHandles(p.x, p.y)
                    }

                    if (isDraggingCorner && bridge.selectedId) {
                        var deltaR = p.x - lastX
                        var newR = Math.max(0, Math.min(250, cornerStartRadius + deltaR))
                        bridge.setCornerRadius(bridge.selectedId, newR)
                        overlayCanvas.requestPaint()
                        return
                    }

                    if (isDraggingTransparency && bridge.selectedId) {
                        var deltaOp = (p.x - lastX) / 200.0
                        var newOp = Math.max(0.0, Math.min(1.0, transparencyStartVal + deltaOp))
                        bridge.setOpacity(newOp)
                        overlayCanvas.requestPaint()
                        return
                    }

                    if (activeHandle !== "" && selObj) {
                        var nx = origX
                        var ny = origY
                        var nw = origW
                        var nh = origH

                        if (activeHandle === "se") {
                            nw = Math.max(8, p.x - origX)
                            nh = Math.max(8, p.y - origY)
                        } else if (activeHandle === "e") {
                            nw = Math.max(8, p.x - origX)
                        } else if (activeHandle === "s") {
                            nh = Math.max(8, p.y - origY)
                        } else if (activeHandle === "nw") {
                            nx = Math.min(origX + origW - 8, p.x)
                            ny = Math.min(origY + origH - 8, p.y)
                            nw = (origX + origW) - nx
                            nh = (origY + origH) - ny
                        } else if (activeHandle === "ne") {
                            ny = Math.min(origY + origH - 8, p.y)
                            nw = Math.max(8, p.x - origX)
                            nh = (origY + origH) - ny
                        } else if (activeHandle === "sw") {
                            nx = Math.min(origX + origW - 8, p.x)
                            nw = (origX + origW) - nx
                            nh = Math.max(8, p.y - origY)
                        } else if (activeHandle === "n") {
                            ny = Math.min(origY + origH - 8, p.y)
                            nh = (origY + origH) - ny
                        } else if (activeHandle === "w") {
                            nx = Math.min(origX + origW - 8, p.x)
                            nw = (origX + origW) - nx
                        }

                        bridge.setObjectBounds(selObj.id, nx, ny, nw, nh)
                        overlayCanvas.requestPaint()
                        return
                    }

                    if (isCreatingArtboard) {
                        artboardCurrentX = p.x
                        artboardCurrentY = p.y
                        overlayCanvas.requestPaint()
                        return
                    }

                    if (isDrawingShape) {
                        shapeCurrentX = p.x
                        shapeCurrentY = p.y
                        overlayCanvas.requestPaint()
                        return
                    }

                    if (isDrawingStroke) {
                        strokePoints.push([p.x, p.y])
                        overlayCanvas.requestPaint()
                        return
                    }

                    if (isMeasuring) {
                        measureCurrentX = p.x
                        measureCurrentY = p.y
                        var m = bridge.measureDistance(measureStartX, measureStartY, p.x, p.y)
                        if (m) {
                            measureDistance = m.distance || 0
                            measureAngle = m.angle || 0
                        }
                        overlayCanvas.requestPaint()
                        return
                    }

                    if (isDraggingGradient) {
                        gradientCurrentX = p.x
                        gradientCurrentY = p.y
                        overlayCanvas.requestPaint()
                        return
                    }

                    if (penIsDraggingHandle && bridge.penActive) {
                        bridge.updatePenLastNodeHandles(p.x, p.y)
                        overlayCanvas.requestPaint()
                        return
                    }

                    if (bridge.shapeBuilderActive) {
                        overlayCanvas.hoverFace = bridge.shapeBuilderHit(p.x, p.y)
                        return
                    }

                    if (!pressed) return
                    if (bridge.tool === "node" && dragActive && dragNode) {
                        if (dragNode.handle) {
                            bridge.moveNodeHandle(dragNode.objId, dragNode.contourIndex, dragNode.nodeIndex, dragNode.handle, p.x, p.y)
                        } else {
                            bridge.moveNode(dragNode.objId, dragNode.contourIndex, dragNode.nodeIndex, p.x, p.y)
                        }
                        overlayCanvas.requestPaint()
                        return
                    }
                    if (dragActive && (bridge.tool === "select" || bridge.tool === "artboard")) {
                        bridge.moveSelected(p.x - lastX, p.y - lastY)
                        lastX = p.x
                        lastY = p.y
                    }
                }

                onReleased: function(mouse) {
                    if (isPanning) {
                        isPanning = false
                        return
                    }
                    if (isDraggingCorner) {
                        isDraggingCorner = false
                        return
                    }
                    if (isDraggingTransparency) {
                        isDraggingTransparency = false
                        return
                    }
                    if (activeHandle !== "") {
                        activeHandle = ""
                        dragActive = false
                        overlayCanvas.requestPaint()
                        return
                    }
                    if (isCreatingArtboard) {
                        isCreatingArtboard = false
                        var ax = Math.min(artboardStartX, artboardCurrentX)
                        var ay = Math.min(artboardStartY, artboardCurrentY)
                        var aw = Math.abs(artboardCurrentX - artboardStartX)
                        var ah = Math.abs(artboardCurrentY - artboardStartY)
                        if (aw >= 30 && ah >= 30) {
                            bridge.createArtboard("", ax, ay, aw, ah, "#ffffff")
                        } else {
                            bridge.createArtboard("", ax, ay, root.artboardPresetW, root.artboardPresetH, "#ffffff")
                        }
                        overlayCanvas.requestPaint()
                        return
                    }

                    if (isDrawingShape) {
                        isDrawingShape = false
                        var sx = Math.min(shapeStartX, shapeCurrentX)
                        var sy = Math.min(shapeStartY, shapeCurrentY)
                        var sw = Math.abs(shapeCurrentX - shapeStartX)
                        var sh = Math.abs(shapeCurrentY - shapeStartY)
                        if (bridge.tool === "text") {
                            if (sw > 30 && sh > 20) {
                                bridge.createTextBounds(sx, sy, sw, sh, "Novo Texto")
                            } else {
                                bridge.createText(shapeStartX, shapeStartY, "Novo Texto")
                            }
                        } else {
                            if (sw > 8 && sh > 8) {
                                bridge.createShapeBounds(bridge.tool, sx, sy, sw, sh)
                            } else {
                                bridge.create_shape(bridge.tool, shapeStartX, shapeStartY)
                            }
                        }
                        overlayCanvas.requestPaint()
                        return
                    }

                    if (isDrawingStroke) {
                        isDrawingStroke = false
                        var swid = (bridge.tool === "brush" || bridge.tool === "pixel_brush") ? Math.max(3, bridge.currentStrokeWidth * 1.5) : bridge.currentStrokeWidth
                        bridge.createFreehandStroke(strokePoints, bridge.currentStroke, swid, "transparent", false, 0.45)
                        strokePoints = []
                        overlayCanvas.requestPaint()
                        return
                    }

                    if (isMeasuring) {
                        isMeasuring = false
                        overlayCanvas.requestPaint()
                        return
                    }

                    if (isDraggingGradient) {
                        isDraggingGradient = false
                        var gdx = gradientCurrentX - gradientStartX
                        var gdy = gradientCurrentY - gradientStartY
                        if (Math.hypot(gdx, gdy) > 10 && bridge.selectedId) {
                            bridge.setObjectGradient(bridge.selectedId, [bridge.currentFill, "#00b4d8"])
                        }
                        overlayCanvas.requestPaint()
                        return
                    }

                    if (penIsDraggingHandle) {
                        penIsDraggingHandle = false
                    }

                    var wasDrag = dragActive
                    dragActive = false
                    dragNode = null
                    if (wasDrag) bridge.snapSelected()
                    overlayCanvas.requestPaint()
                }

                onDoubleClicked: function(mouse) {
                    if (bridge.tool === "pen") bridge.penFinish()
                }

                onWheel: function(wheel) {
                    var oldZoom = bridge.zoom
                    var factor = wheel.angleDelta.y > 0 ? 1.15 : 1 / 1.15
                    var newZoom = Math.max(0.05, Math.min(8.0, oldZoom * factor))
                    root.panX = wheel.x - (wheel.x - root.panX) * (newZoom / oldZoom)
                    root.panY = wheel.y - (wheel.y - root.panY) * (newZoom / oldZoom)
                    bridge.setZoom(newZoom)
                }
            }
        }

        // =========================================================
        // RIGHT SIDEBAR: Studio Panels (3 Stacked Docks)
        // =========================================================
        RightStudio {
            id: rightStudio
            panX: root.panX
            panY: root.panY
            canvasWidth: canvasArea.width
            canvasHeight: canvasArea.height
            totalDocWidth: root.totalDocWidth
            totalDocHeight: root.totalDocHeight
            onPanRequested: function(nx, ny) {
                root.panX = nx
                root.panY = ny
            }
        }
    }

    // -------------------------------------------------------------
    // Bottom Status Bar
    // -------------------------------------------------------------
    footer: StatusBar {
        id: statusBar
        canvasHintText: root.canvasHintText
    }

    // -------------------------------------------------------------
    // Modals / Dialogs (Document Setup, App Settings, Help)
    // -------------------------------------------------------------
    NewDocumentDialog {
        id: newDocDialog
    }

    DocumentSetupDialog {
        id: docSetupDialog
    }

    AppSettingsDialog {
        id: appSettingsDialog
    }

    HelpShortcutsDialog {
        id: helpDialog
    }

    FileDialog {
        id: placeFileDialog
        title: "Selecionar Imagem"
        nameFilters: ["Imagens (*.png *.jpg *.jpeg *.svg *.webp)", "Todos os arquivos (*)"]
        onAccepted: {
            var path = selectedFile.toString().replace("file://", "")
            bridge.placeImage(path, 100, 100, 360, 240)
        }
    }
}
