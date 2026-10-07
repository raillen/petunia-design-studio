import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs
import "../components"
import "../dialogs"

Rectangle {
    id: canvasArea
    property var root: canvasArea

    property real panX: 200
    property real panY: 100
    property real totalDocWidth: 6000
    property real totalDocHeight: 4000
    property color bgCanvasWorkspace: "#1e1e20"
    property color accentCyan: "#00b4d8"
    property color accentHover: "#48cae4"
    property color textSecondary: "#a1a1a6"
    property color textDim: "#6c6c70"
    property real artboardPresetW: 1920
    property real artboardPresetH: 1080
    property bool spacePressed: false
    property string activeShapeType: "rectangle"

    function selectedObject() {
        if (!bridge || !bridge.objects) return null
        var objs = bridge.objects
        var selId = bridge.selectedId || ""
        for (var i = 0; i < objs.length; i++) {
            if (objs[i].id === selId) return objs[i]
        }
        return null
    }

    function centerActiveArtboard() {
        if (!bridge) return
        panX = Math.max(40, (canvasArea.width - bridge.activeArtboardWidth * bridge.zoom) / 2)
        panY = Math.max(40, (canvasArea.height - bridge.activeArtboardHeight * bridge.zoom) / 2)
    }

    function drawPathData(ctx, contours) {
        if (!contours) return
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

    Connections {
        target: (typeof bridge !== "undefined" && bridge !== null) ? bridge : null
        function onActiveArtboardIndexChanged() {
            canvasArea.centerActiveArtboard()
        }
        function onViewportFocusRequested(ax, ay, az) {
            canvasArea.panX = ax
            canvasArea.panY = ay
            if (bridge) bridge.setZoom(az)
        }
        function onNewDocumentRequested() { newDocDialog.open() }
        function onDocumentSetupRequested() { docSetupDialog.open() }
        function onAppSettingsRequested() { appSettingsDialog.open() }
        function onHelpRequested() { helpDialog.open() }
        function onPlaceImageRequested() { placeFileDialog.open() }
    }
    color: root.bgCanvasWorkspace
    clip: true

            Item {
                id: docViewport
                x: root.panX
                y: root.panY
                width: root.totalDocWidth
                height: root.totalDocHeight
                scale: bridge ? bridge.zoom : 1.0
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
                // 1b. Export Persona Slices
                // -------------------------------------------------------------
                Repeater {
                    model: bridge.slices
                    delegate: Item {
                        x: modelData.x
                        y: modelData.y
                        width: modelData.width
                        height: modelData.height
                        z: 10
                        visible: bridge.persona === "export" || bridge.tool === "slice" || bridge.tool === "slice_select"

                        // Slice Border Box
                        Rectangle {
                            anchors.fill: parent
                            color: "transparent"
                            border.color: "#f97316"
                            border.width: Math.max(1.5, 2.0 / bridge.zoom)

                            // Slice Tag Header
                            Rectangle {
                                anchors.bottom: parent.top
                                anchors.left: parent.left
                                anchors.bottomMargin: 2
                                height: Math.max(16, 18 / bridge.zoom)
                                width: sliceHeaderRow.implicitWidth + 8
                                radius: 2
                                color: "#f97316"

                                RowLayout {
                                    id: sliceHeaderRow
                                    anchors.centerIn: parent
                                    spacing: 4
                                    Text {
                                        text: Phosphor.knife
                                        font.family: "Phosphor"
                                        font.pixelSize: Math.max(8, Math.round(9 / bridge.zoom))
                                        color: "#ffffff"
                                    }
                                    Text {
                                        text: modelData.name + " (" + Math.round(modelData.width) + "x" + Math.round(modelData.height) + " " + (modelData.format || "PNG") + ")"
                                        font.pixelSize: Math.max(8, Math.round(9 / bridge.zoom))
                                        font.bold: true
                                        color: "#ffffff"
                                    }
                                    // Export Button inside tag
                                    Rectangle {
                                        width: Math.max(12, 14 / bridge.zoom)
                                        height: Math.max(12, 14 / bridge.zoom)
                                        radius: 2
                                        color: "#ea580c"
                                        Text {
                                            anchors.centerIn: parent
                                            text: Phosphor.downloadSimple
                                            font.family: "Phosphor"
                                            font.pixelSize: Math.max(7, Math.round(8 / bridge.zoom))
                                            color: "#ffffff"
                                        }
                                        MouseArea {
                                            anchors.fill: parent
                                            cursorShape: Qt.PointingHandCursor
                                            onClicked: bridge.exportSlice(modelData.id)
                                        }
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
                                ctx.save()

                                if (o.type === "path" && o.path)
                                    ctx.translate(-o.x, -o.y)

                                ctx.globalAlpha = (o.opacity !== undefined && o.opacity !== null) ? o.opacity : 1.0

                                if (o.blendMode && o.blendMode !== "normal") {
                                    var bm = o.blendMode.toLowerCase()
                                    if (bm === "multiply") ctx.globalCompositeOperation = "multiply"
                                    else if (bm === "screen") ctx.globalCompositeOperation = "screen"
                                    else if (bm === "overlay") ctx.globalCompositeOperation = "overlay"
                                    else if (bm === "darken") ctx.globalCompositeOperation = "darken"
                                    else if (bm === "lighten") ctx.globalCompositeOperation = "lighten"
                                    else if (bm === "destination-out" || bm === "eraser") ctx.globalCompositeOperation = "destination-out"
                                    else ctx.globalCompositeOperation = "source-over"
                                } else {
                                    ctx.globalCompositeOperation = "source-over"
                                }

                                if (o.rotation) {
                                    var cx = width / 2
                                    var cy = height / 2
                                    ctx.translate(cx, cy)
                                    ctx.rotate((o.rotation * Math.PI) / 180)
                                    ctx.translate(-cx, -cy)
                                }

                                ctx.lineCap = o.strokeCap || "round"
                                ctx.lineJoin = o.strokeJoin || "round"
                                if (o.strokeDash && o.strokeDash.length > 0) {
                                    ctx.setLineDash(o.strokeDash)
                                } else {
                                    ctx.setLineDash([])
                                }

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
                                    var fSize = o.fontSize || 14
                                    var fFam = o.fontFamily || "Inter, sans-serif"
                                    var fWeight = (o.fontWeight === "bold" || o.fontWeight === "700") ? "bold " : ""
                                    var fStyle = o.fontStyle === "italic" ? "italic " : ""
                                    ctx.font = fStyle + fWeight + fSize + "px " + fFam
                                    ctx.textBaseline = "middle"
                                    var tAlign = o.textAlign || "left"
                                    ctx.textAlign = tAlign === "center" ? "center" : (tAlign === "right" ? "right" : "left")
                                    var txt = o.text !== undefined ? o.text : (o.name || "Texto")
                                    var textX = tAlign === "center" ? width / 2 : (tAlign === "right" ? width - 4 : 4)
                                    ctx.fillText(txt, textX, height / 2)
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
                                ctx.restore()
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

                        // 6. Shape Creation Live Drag Preview
                        if (canvasMouse.isDrawingShape && (bridge.tool === "rectangle" || bridge.tool === "ellipse" || bridge.tool === "triangle" || bridge.tool === "star" || bridge.tool === "polygon" || bridge.tool === "diamond" || bridge.tool === "arrow" || bridge.tool === "heart" || bridge.tool === "cog" || bridge.tool === "text")) {
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
                            } else if (bridge.tool === "triangle") {
                                ctx.moveTo(sx + sw / 2, sy)
                                ctx.lineTo(sx + sw, sy + sh)
                                ctx.lineTo(sx, sy + sh)
                                ctx.closePath()
                            } else if (bridge.tool === "star") {
                                var starCx = sx + sw / 2
                                var starCy = sy + sh / 2
                                var starRx = sw / 2
                                var starRy = sh / 2
                                var sPoints = Math.max(3, bridge.starPoints || 5)
                                var sInner = bridge.starInnerRadius || 0.45
                                var sTotal = sPoints * 2
                                for (var sti = 0; sti < sTotal; sti++) {
                                    var stAngle = -Math.PI / 2 + sti * (Math.PI / sPoints)
                                    var stRadX = sti % 2 === 0 ? starRx : starRx * sInner
                                    var stRadY = sti % 2 === 0 ? starRy : starRy * sInner
                                    var stpx = starCx + stRadX * Math.cos(stAngle)
                                    var stpy = starCy + stRadY * Math.sin(stAngle)
                                    if (sti === 0) ctx.moveTo(stpx, stpy)
                                    else ctx.lineTo(stpx, stpy)
                                }
                                ctx.closePath()
                            } else if (bridge.tool === "polygon") {
                                var polyCx = sx + sw / 2
                                var polyCy = sy + sh / 2
                                var pSides = Math.max(3, bridge.polygonSides || 6)
                                for (var pli = 0; pli < pSides; pli++) {
                                    var plAngle = -Math.PI / 2 + pli * (2 * Math.PI / pSides)
                                    var plpx = polyCx + (sw / 2) * Math.cos(plAngle)
                                    var plpy = polyCy + (sh / 2) * Math.sin(plAngle)
                                    if (pli === 0) ctx.moveTo(plpx, plpy)
                                    else ctx.lineTo(plpx, plpy)
                                }
                                ctx.closePath()
                            } else if (bridge.tool === "diamond") {
                                ctx.moveTo(sx + sw / 2, sy)
                                ctx.lineTo(sx + sw, sy + sh / 2)
                                ctx.lineTo(sx + sw / 2, sy + sh)
                                ctx.lineTo(sx, sy + sh / 2)
                                ctx.closePath()
                            } else if (bridge.tool === "arrow") {
                                var hLen = sw * 0.4
                                var sH = sh * 0.4
                                var yTop = sy + (sh - sH) / 2
                                var yBot = yTop + sH
                                ctx.moveTo(sx, yTop)
                                ctx.lineTo(sx + sw - hLen, yTop)
                                ctx.lineTo(sx + sw - hLen, sy)
                                ctx.lineTo(sx + sw, sy + sh / 2)
                                ctx.lineTo(sx + sw - hLen, sy + sh)
                                ctx.lineTo(sx + sw - hLen, yBot)
                                ctx.lineTo(sx, yBot)
                                ctx.closePath()
                            } else if (bridge.tool === "heart") {
                                var hcx = sx + sw / 2
                                ctx.moveTo(hcx, sy + sh * 0.25)
                                ctx.bezierCurveTo(sx + sw * 0.25, sy, sx, sy + sh * 0.35, hcx, sy + sh)
                                ctx.bezierCurveTo(sx + sw, sy + sh * 0.35, sx + sw * 0.75, sy, hcx, sy + sh * 0.25)
                                ctx.closePath()
                            } else if (bridge.tool === "cog") {
                                var cogCx = sx + sw / 2
                                var cogCy = sy + sh / 2
                                var cogRo = Math.min(sw, sh) / 2
                                var cogDepth = (bridge && bridge.cogToothDepth) ? bridge.cogToothDepth : 0.3
                                var cogRr = cogRo * Math.max(0.2, 1.0 - cogDepth)
                                var cogTeeth = Math.max(3, (bridge && bridge.cogTeeth) ? bridge.cogTeeth : 8)
                                var cogStep = 2 * Math.PI / cogTeeth
                                for (var cti = 0; cti < cogTeeth; cti++) {
                                    var cth0 = -Math.PI / 2 + cti * cogStep
                                    var cth1 = cth0 + 0.25 * cogStep
                                    var cth2 = cth0 + 0.65 * cogStep
                                    var cth3 = cth0 + 0.85 * cogStep
                                    var cp0x = cogCx + cogRr * Math.cos(cth0)
                                    var cp0y = cogCy + cogRr * Math.sin(cth0)
                                    var cp1x = cogCx + cogRo * Math.cos(cth1)
                                    var cp1y = cogCy + cogRo * Math.sin(cth1)
                                    var cp2x = cogCx + cogRo * Math.cos(cth2)
                                    var cp2y = cogCy + cogRo * Math.sin(cth2)
                                    var cp3x = cogCx + cogRr * Math.cos(cth3)
                                    var cp3y = cogCy + cogRr * Math.sin(cth3)
                                    if (cti === 0) ctx.moveTo(cp0x, cp0y)
                                    else ctx.lineTo(cp0x, cp0y)
                                    ctx.lineTo(cp1x, cp1y)
                                    ctx.lineTo(cp2x, cp2y)
                                    ctx.lineTo(cp3x, cp3y)
                                }
                                ctx.closePath()
                            } else if (bridge.tool === "text") {
                                ctx.rect(sx, sy, sw, sh)
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

                        // 12. Slice Live Drag Preview
                        if (canvasMouse.isCreatingSlice) {
                            var slx = Math.min(canvasMouse.sliceStartX, canvasMouse.sliceCurrentX)
                            var sly = Math.min(canvasMouse.sliceStartY, canvasMouse.sliceCurrentY)
                            var slw = Math.abs(canvasMouse.sliceCurrentX - canvasMouse.sliceStartX)
                            var slh = Math.abs(canvasMouse.sliceCurrentY - canvasMouse.sliceStartY)

                            ctx.fillStyle = "rgba(249, 115, 22, 0.15)"
                            ctx.fillRect(slx, sly, slw, slh)

                            ctx.strokeStyle = "#f97316"
                            ctx.lineWidth = 1.5
                            ctx.setLineDash([5, 4])
                            ctx.strokeRect(slx, sly, slw, slh)
                            ctx.setLineDash([])

                            var slBadgeW = 96
                            var slBadgeH = 20
                            var slbx = slx + slw / 2 - slBadgeW / 2
                            var slby = sly + slh + 8
                            ctx.fillStyle = "#1e1e20"
                            ctx.fillRect(slbx, slby, slBadgeW, slBadgeH)
                            ctx.strokeStyle = "#f97316"
                            ctx.lineWidth = 1
                            ctx.strokeRect(slbx, slby, slBadgeW, slBadgeH)
                            ctx.fillStyle = "#ffffff"
                            ctx.font = "10px Inter, sans-serif"
                            ctx.textAlign = "center"
                            ctx.textBaseline = "middle"
                            ctx.fillText("Fatia: " + Math.round(slw) + " x " + Math.round(slh), slbx + slBadgeW / 2, slby + slBadgeH / 2)
                        }

                        // 13. Pixel Marquee Selection Preview
                        if (canvasMouse.isMarqueeSelecting) {
                            var mx = Math.min(canvasMouse.marqueeStartX, canvasMouse.marqueeCurrentX)
                            var my = Math.min(canvasMouse.marqueeStartY, canvasMouse.marqueeCurrentY)
                            var mw = Math.abs(canvasMouse.marqueeCurrentX - canvasMouse.marqueeStartX)
                            var mh = Math.abs(canvasMouse.marqueeCurrentY - canvasMouse.marqueeStartY)

                            ctx.strokeStyle = "#ffffff"
                            ctx.lineWidth = 1.0
                            ctx.setLineDash([4, 4])
                            ctx.strokeRect(mx, my, mw, mh)
                            ctx.setLineDash([])
                        }

                        // 14. Knife Tool Cut Line Preview
                        if (canvasMouse.isCuttingKnife) {
                            var kx1 = canvasMouse.knifeStartX
                            var ky1 = canvasMouse.knifeStartY
                            var kx2 = canvasMouse.knifeCurrentX
                            var ky2 = canvasMouse.knifeCurrentY
                            var kdx = kx2 - kx1
                            var kdy = ky2 - ky1
                            var kdist = Math.sqrt(kdx * kdx + kdy * kdy)

                            ctx.strokeStyle = "#ef4444"
                            ctx.lineWidth = 2.0
                            ctx.beginPath()
                            ctx.moveTo(kx1, ky1)
                            ctx.lineTo(kx2, ky2)
                            ctx.stroke()

                            var ktLen = 8
                            if (kdist > 1) {
                                var kpx = -kdy / kdist * ktLen
                                var kpy = kdx / kdist * ktLen
                                ctx.beginPath()
                                ctx.moveTo(kx1 - kpx, ky1 - kpy); ctx.lineTo(kx1 + kpx, ky1 + kpy)
                                ctx.moveTo(kx2 - kpx, ky2 - kpy); ctx.lineTo(kx2 + kpx, ky2 + kpy)
                                ctx.stroke()
                            }

                            var kBadgeW = 100
                            var kBadgeH = 20
                            var kbx = (kx1 + kx2) / 2 - kBadgeW / 2
                            var kby = (ky1 + ky2) / 2 - 26
                            ctx.fillStyle = "#1e1e20"
                            ctx.fillRect(kbx, kby, kBadgeW, kBadgeH)
                            ctx.strokeStyle = "#ef4444"
                            ctx.lineWidth = 1
                            ctx.strokeRect(kbx, kby, kBadgeW, kBadgeH)
                            ctx.fillStyle = "#ffffff"
                            ctx.font = "10px Inter, sans-serif"
                            ctx.textAlign = "center"
                            ctx.textBaseline = "middle"
                            ctx.fillText("Corte: " + Math.round(kdist) + " px", kbx + kBadgeW / 2, kby + kBadgeH / 2)
                        }

                        // 15. Pixel Lasso Polygon Preview
                        if (canvasMouse.isLassoSelecting && canvasMouse.lassoPoints.length > 1) {
                            var lpts = canvasMouse.lassoPoints
                            ctx.strokeStyle = "#38bdf8"
                            ctx.lineWidth = 1.5
                            ctx.setLineDash([4, 4])
                            ctx.beginPath()
                            ctx.moveTo(lpts[0][0], lpts[0][1])
                            for (var lpi = 1; lpi < lpts.length; lpi++) {
                                ctx.lineTo(lpts[lpi][0], lpts[lpi][1])
                            }
                            ctx.stroke()
                            ctx.setLineDash([])
                        }

                        // 16. Clone Stamp Target Reticle Preview
                        if (bridge && bridge.tool === "pixel_clone" && bridge.cloneSourceSet) {
                            var csx = bridge.cloneSourceX
                            var csy = bridge.cloneSourceY
                            ctx.strokeStyle = "#38bdf8"
                            ctx.lineWidth = 1.5
                            ctx.beginPath()
                            ctx.arc(csx, csy, 10, 0, Math.PI * 2)
                            ctx.stroke()
                            ctx.beginPath()
                            ctx.moveTo(csx - 14, csy); ctx.lineTo(csx + 14, csy)
                            ctx.moveTo(csx, csy - 14); ctx.lineTo(csx, csy + 14)
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

                property bool isCreatingSlice: false
                property real sliceStartX: 0
                property real sliceStartY: 0
                property real sliceCurrentX: 0
                property real sliceCurrentY: 0

                property bool isCuttingKnife: false
                property real knifeStartX: 0
                property real knifeStartY: 0
                property real knifeCurrentX: 0
                property real knifeCurrentY: 0

                property bool isLassoSelecting: false
                property var lassoPoints: []

                property bool isMarqueeSelecting: false
                property real marqueeStartX: 0
                property real marqueeStartY: 0
                property real marqueeCurrentX: 0
                property real marqueeCurrentY: 0

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
                    if (bridge.tool === "artboard" || bridge.tool === "rectangle" || bridge.tool === "ellipse" || bridge.tool === "triangle" || bridge.tool === "star" || bridge.tool === "polygon" || bridge.tool === "diamond" || bridge.tool === "arrow" || bridge.tool === "heart" || bridge.tool === "cog" || bridge.tool === "measure" || bridge.tool === "pixel_select" || bridge.tool === "pixel_lasso" || bridge.tool === "knife" || bridge.tool === "slice") return Qt.CrossCursor
                    if (bridge.tool === "zoom" || bridge.tool === "pixel_fill" || bridge.tool === "slice_select") return Qt.PointingHandCursor
                    if (bridge.tool === "node" || bridge.tool === "pen") return Qt.CrossCursor
                    if (bridge.tool === "pencil" || bridge.tool === "brush" || bridge.tool === "pixel_brush" || bridge.tool === "pixel_pencil" || bridge.tool === "pixel_eraser" || bridge.tool === "pixel_clone" || bridge.tool === "pixel_dodge" || bridge.tool === "pixel_burn" || bridge.tool === "pixel_smudge" || bridge.tool === "pixel_blur") return Qt.CrossCursor
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

                    if (bridge.tool === "knife") {
                        isCuttingKnife = true
                        knifeStartX = p.x
                        knifeStartY = p.y
                        knifeCurrentX = p.x
                        knifeCurrentY = p.y
                        overlayCanvas.requestPaint()
                        return
                    }

                    if (bridge.tool === "pixel_lasso") {
                        isLassoSelecting = true
                        lassoPoints = [[p.x, p.y]]
                        overlayCanvas.requestPaint()
                        return
                    }

                    if (bridge.tool === "pixel_clone") {
                        if (mouse.modifiers & Qt.AltModifier) {
                            bridge.setCloneSource(p.x, p.y)
                            overlayCanvas.requestPaint()
                            return
                        }
                        isDrawingStroke = true
                        strokePoints = [[p.x, p.y]]
                        overlayCanvas.requestPaint()
                        return
                    }

                    if (bridge.tool === "pixel_dodge" || bridge.tool === "pixel_burn" || bridge.tool === "pixel_smudge" || bridge.tool === "pixel_blur") {
                        isDrawingStroke = true
                        strokePoints = [[p.x, p.y]]
                        overlayCanvas.requestPaint()
                        return
                    }

                    if (bridge.tool === "slice") {
                        isCreatingSlice = true
                        sliceStartX = p.x
                        sliceStartY = p.y
                        sliceCurrentX = p.x
                        sliceCurrentY = p.y
                        overlayCanvas.requestPaint()
                        return
                    }

                    if (bridge.tool === "slice_select") {
                        for (var si = bridge.slices.length - 1; si >= 0; si--) {
                            var slc = bridge.slices[si]
                            if (p.x >= slc.x && p.x <= slc.x + slc.width && p.y >= slc.y && p.y <= slc.y + slc.height) {
                                bridge.selectSlice(slc.id)
                                break
                            }
                        }
                        return
                    }

                    if (bridge.tool === "pixel_select") {
                        isMarqueeSelecting = true
                        marqueeStartX = p.x
                        marqueeStartY = p.y
                        marqueeCurrentX = p.x
                        marqueeCurrentY = p.y
                        overlayCanvas.requestPaint()
                        return
                    }

                    if (bridge.tool === "pixel_fill") {
                        bridge.floodFill(p.x, p.y)
                        return
                    }

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

                    if (bridge.tool === "rectangle" || bridge.tool === "ellipse" || bridge.tool === "triangle" || bridge.tool === "star" || bridge.tool === "polygon" || bridge.tool === "diamond" || bridge.tool === "arrow" || bridge.tool === "heart" || bridge.tool === "cog" || bridge.tool === "text") {
                        isDrawingShape = true
                        shapeStartX = p.x
                        shapeStartY = p.y
                        shapeCurrentX = p.x
                        shapeCurrentY = p.y
                        overlayCanvas.requestPaint()
                        return
                    }

                    if (bridge.tool === "pencil" || bridge.tool === "brush" || bridge.tool === "pixel_brush" || bridge.tool === "pixel_pencil" || bridge.tool === "pixel_eraser") {
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

                    if (isCuttingKnife) {
                        knifeCurrentX = p.x
                        knifeCurrentY = p.y
                        overlayCanvas.requestPaint()
                        return
                    }

                    if (isLassoSelecting) {
                        lassoPoints.push([p.x, p.y])
                        overlayCanvas.requestPaint()
                        return
                    }

                    if (isCreatingSlice) {
                        sliceCurrentX = p.x
                        sliceCurrentY = p.y
                        overlayCanvas.requestPaint()
                        return
                    }

                    if (isMarqueeSelecting) {
                        marqueeCurrentX = p.x
                        marqueeCurrentY = p.y
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
                    if (isCuttingKnife) {
                        isCuttingKnife = false
                        bridge.knifeCut(knifeStartX, knifeStartY, knifeCurrentX, knifeCurrentY)
                        overlayCanvas.requestPaint()
                        return
                    }
                    if (isLassoSelecting) {
                        isLassoSelecting = false
                        bridge.lassoSelect(lassoPoints, (mouse.modifiers & Qt.ShiftModifier))
                        lassoPoints = []
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

                    if (isCreatingSlice) {
                        isCreatingSlice = false
                        var slx = Math.min(sliceStartX, sliceCurrentX)
                        var sly = Math.min(sliceStartY, sliceCurrentY)
                        var slw = Math.abs(sliceCurrentX - sliceStartX)
                        var slh = Math.abs(sliceCurrentY - sliceStartY)
                        if (slw >= 15 && slh >= 15) {
                            bridge.createSlice("", slx, sly, slw, slh, "PNG", 1.0)
                        } else {
                            bridge.createSlice("", slx, sly, 256, 256, "PNG", 1.0)
                        }
                        overlayCanvas.requestPaint()
                        return
                    }

                    if (isMarqueeSelecting) {
                        isMarqueeSelecting = false
                        var mx = Math.min(marqueeStartX, marqueeCurrentX)
                        var my = Math.min(marqueeStartY, marqueeCurrentY)
                        var mw = Math.abs(marqueeCurrentX - marqueeStartX)
                        var mh = Math.abs(marqueeCurrentY - marqueeStartY)
                        if (mw > 4 && mh > 4) {
                            for (var oi = 0; oi < bridge.renderObjects.length; oi++) {
                                var ro = bridge.renderObjects[oi]
                                if (ro.x < mx + mw && ro.x + ro.width > mx && ro.y < my + mh && ro.y + ro.height > my) {
                                    bridge.select(ro.id)
                                    break
                                }
                            }
                        }
                        overlayCanvas.requestPaint()
                        return
                    }

                    if (isDrawingStroke) {
                        isDrawingStroke = false
                        if (bridge.tool === "pixel_eraser") {
                            var swid_er = Math.max(4, bridge.currentStrokeWidth * 2)
                            var er_id = bridge.createFreehandStroke(strokePoints, "#ffffff", swid_er, "transparent", false, 0.45, "destination-out")
                            bridge.setBlendMode(er_id, "destination-out")
                        } else if (bridge.tool === "pixel_pencil") {
                            bridge.createFreehandStroke(strokePoints, bridge.currentStroke, 1.0, "transparent", false, 0.0, "normal")
                        } else if (bridge.tool === "pixel_dodge") {
                            bridge.createFreehandStroke(strokePoints, "#ffffff", Math.max(4, bridge.currentStrokeWidth * 2), "transparent", false, 0.45, "screen")
                        } else if (bridge.tool === "pixel_burn") {
                            bridge.createFreehandStroke(strokePoints, "#000000", Math.max(4, bridge.currentStrokeWidth * 2), "transparent", false, 0.45, "multiply")
                        } else if (bridge.tool === "pixel_smudge" || bridge.tool === "pixel_blur") {
                            bridge.createFreehandStroke(strokePoints, bridge.currentStroke, Math.max(4, bridge.currentStrokeWidth * 2), "transparent", false, 0.45, "normal")
                        } else {
                            var swid = (bridge.tool === "brush" || bridge.tool === "pixel_brush" || bridge.tool === "pixel_clone") ? Math.max(3, bridge.currentStrokeWidth * 1.5) : bridge.currentStrokeWidth
                            bridge.createFreehandStroke(strokePoints, bridge.currentStroke, swid, "transparent", false, 0.45, "normal")
                        }
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

            // Keyboard Shortcuts
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
            Shortcut { sequence: "B"; onActivated: bridge.setTool(bridge.persona === "pixel" ? "pixel_brush" : "pencil") }
            Shortcut { sequence: "M"; onActivated: bridge.setTool(bridge.persona === "pixel" ? "pixel_select" : "rectangle") }
            Shortcut { sequence: "O"; onActivated: bridge.setTool("ellipse") }
            Shortcut { sequence: "T"; onActivated: bridge.setTool("text") }
            Shortcut { sequence: "G"; onActivated: bridge.setTool(bridge.persona === "pixel" ? "pixel_fill" : "fill") }
            Shortcut { sequence: "Y"; onActivated: bridge.setTool("transparency") }
            Shortcut { sequence: "X"; onActivated: bridge.setTool("crop") }
            Shortcut { sequence: "K"; onActivated: bridge.setTool("knife") }
            Shortcut { sequence: "L"; onActivated: bridge.setTool(bridge.persona === "pixel" ? "pixel_lasso" : "select") }
            Shortcut { sequence: "I"; onActivated: bridge.setTool("eyedropper") }
            Shortcut { sequence: "H"; onActivated: bridge.setTool("hand") }
            Shortcut { sequence: "Z"; onActivated: bridge.setTool("zoom") }
            Shortcut { sequence: "S"; onActivated: bridge.setTool(bridge.persona === "pixel" ? "pixel_clone" : "slice") }
            Shortcut { sequence: "Return"; onActivated: bridge.penFinish() }
            Shortcut { sequence: "Escape"; onActivated: bridge.penCancel() }
        }

