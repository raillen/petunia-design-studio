import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components"

Rectangle {
    id: colourPanel
    Layout.fillWidth: true
    Layout.fillHeight: true

    property color bgPanelBody: "#28282a"
    property color bgField: "#181819"
    property color borderSubtle: "#333335"
    property color textPrimary: "#f2f2f5"
    property color textSecondary: "#a1a1a6"
    property color textDim: "#6c6c70"

    property real colorHue: 310.0
    property real colorSat: 0.75
    property real colorVal: 0.50
    property string activeHex: bridge ? bridge.currentFill : "#e120a5"
    property bool editingStroke: false

    function hsvToHex(h, s, v) {
        var c = v * s
        var x = c * (1 - Math.abs((h / 60) % 2 - 1))
        var m = v - c
        var r = 0, g = 0, b = 0
        if (h < 60) { r = c; g = x; b = 0 }
        else if (h < 120) { r = x; g = c; b = 0 }
        else if (h < 180) { r = 0; g = c; b = x }
        else if (h < 240) { r = 0; g = x; b = c }
        else if (h < 300) { r = x; g = 0; b = c }
        else { r = c; g = 0; b = x }
        var ri = Math.round((r + m) * 255)
        var gi = Math.round((g + m) * 255)
        var bi = Math.round((b + m) * 255)
        var toHex = function(n) {
            var hex = n.toString(16)
            return hex.length === 1 ? "0" + hex : hex
        }
        return "#" + toHex(ri) + toHex(gi) + toHex(bi)
    }

    function updateColorFromWheel() {
        var hex = hsvToHex(colorHue, colorSat, colorVal)
        activeHex = hex.toUpperCase()
        if (bridge) {
            if (editingStroke) {
                bridge.setStrokeColor(hex)
            } else {
                bridge.setFillColor(hex)
            }
        }
    }

    Connections {
        target: bridge
        function onSelectionChanged() {
            if (bridge) {
                colourPanel.activeHex = bridge.currentFill
                colorWheelCanvas.requestPaint()
            }
        }
    }

    color: bgPanelBody

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 8
        spacing: 6

        // Top row: Color indicators (Fill / Stroke), eyedropper, lock
        RowLayout {
            Layout.fillWidth: true
            spacing: 6

            Item {
                width: 32
                height: 20
                Rectangle {
                    x: 10; y: 2; width: 14; height: 14; radius: 7
                    color: "transparent"
                    border.color: bridge ? bridge.currentStroke : "#333333"
                    border.width: 2
                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: colourPanel.editingStroke = true
                    }
                }
                Rectangle {
                    x: 2; y: 0; width: 16; height: 16; radius: 8
                    color: bridge ? bridge.currentFill : "#e120a5"
                    border.color: "#ffffff"
                    border.width: 1
                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: colourPanel.editingStroke = false
                    }
                }
            }

            Text {
                text: colourPanel.editingStroke ? "Traço" : "Preenchimento"
                color: colourPanel.textSecondary
                font.pixelSize: 10
            }

            Item { Layout.fillWidth: true }

            Text {
                text: Phosphor.eyedropper
                font.family: "Phosphor"
                font.pixelSize: 13
                color: colourPanel.textSecondary
                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: if (bridge) bridge.setTool("eyedropper")
                }
            }

            Text {
                text: Phosphor.lock
                font.family: "Phosphor"
                font.pixelSize: 12
                color: colourPanel.textDim
            }
        }

        // Center Row: Color Wheel + Quick Swatches Column
        RowLayout {
            Layout.fillWidth: true
            Layout.alignment: Qt.AlignHCenter
            spacing: 8

            // Interactive HSV Color Wheel Canvas
            Canvas {
                id: colorWheelCanvas
                Layout.preferredWidth: 124
                Layout.preferredHeight: 124
                Layout.alignment: Qt.AlignHCenter
                renderStrategy: Canvas.Immediate

                Component.onCompleted: requestPaint()

                onPaint: {
                    var ctx = getContext("2d")
                    ctx.clearRect(0, 0, width, height)
                    var cx = width / 2
                    var cy = height / 2
                    var rOut = width / 2 - 2
                    var rIn = rOut - 13

                    // Draw Rainbow Donut
                    for (var a = 0; a < 360; a += 2) {
                        var rad1 = (a - 1) * Math.PI / 180
                        var rad2 = (a + 2) * Math.PI / 180
                        ctx.beginPath()
                        ctx.arc(cx, cy, (rOut + rIn) / 2, rad1, rad2, false)
                        ctx.strokeStyle = "hsl(" + a + ", 100%, 50%)"
                        ctx.lineWidth = (rOut - rIn)
                        ctx.stroke()
                    }

                    // Draw Inner Triangle (HSV Sat/Val)
                    ctx.save()
                    ctx.beginPath()
                    var triR = rIn - 4
                    var triA = (colourPanel.colorHue - 90) * Math.PI / 180
                    var p1 = [cx + triR * Math.cos(triA), cy + triR * Math.sin(triA)]
                    var p2 = [cx + triR * Math.cos(triA + 2.0944), cy + triR * Math.sin(triA + 2.0944)]
                    var p3 = [cx + triR * Math.cos(triA + 4.1888), cy + triR * Math.sin(triA + 4.1888)]
                    ctx.moveTo(p1[0], p1[1])
                    ctx.lineTo(p2[0], p2[1])
                    ctx.lineTo(p3[0], p3[1])
                    ctx.closePath()

                    var grad = ctx.createLinearGradient(p2[0], p2[1], p1[0], p1[1])
                    grad.addColorStop(0, "#000000")
                    grad.addColorStop(0.5, "#808080")
                    grad.addColorStop(1, "hsl(" + colourPanel.colorHue + ", 100%, 50%)")
                    ctx.fillStyle = grad
                    ctx.fill()
                    ctx.strokeStyle = "#404044"
                    ctx.lineWidth = 1
                    ctx.stroke()
                    ctx.restore()

                    // Draw Hue ring marker
                    var hRad = colourPanel.colorHue * Math.PI / 180
                    var mx = cx + ((rOut + rIn) / 2) * Math.cos(hRad)
                    var my = cy + ((rOut + rIn) / 2) * Math.sin(hRad)
                    ctx.beginPath()
                    ctx.arc(mx, my, 4.5, 0, Math.PI * 2)
                    ctx.fillStyle = "#ffffff"
                    ctx.fill()
                    ctx.strokeStyle = "#111111"
                    ctx.lineWidth = 1.5
                    ctx.stroke()

                    // Draw Saturation/Lightness cursor inside triangle
                    var pickX = cx + (p1[0] - cx) * colourPanel.colorSat * 0.7
                    var pickY = cy + (p1[1] - cy) * colourPanel.colorSat * 0.7
                    ctx.beginPath()
                    ctx.arc(pickX, pickY, 4, 0, Math.PI * 2)
                    ctx.fillStyle = colourPanel.activeHex
                    ctx.fill()
                    ctx.strokeStyle = "#ffffff"
                    ctx.lineWidth = 1.5
                    ctx.stroke()
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.CrossCursor
                    onPressed: updateFromPoint(mouse)
                    onPositionChanged: if (pressed) updateFromPoint(mouse)

                    function updateFromPoint(mouse) {
                        var cx = width / 2
                        var cy = height / 2
                        var dx = mouse.x - cx
                        var dy = mouse.y - cy
                        var dist = Math.sqrt(dx * dx + dy * dy)
                        var rIn = width / 2 - 15

                        if (dist >= rIn - 8) {
                            var angle = Math.atan2(dy, dx) * 180 / Math.PI
                            if (angle < 0) angle += 360
                            colourPanel.colorHue = Math.round(angle)
                        } else {
                            colourPanel.colorSat = Math.max(0.1, Math.min(1.0, dist / rIn))
                            colourPanel.colorVal = 0.5 + (0.5 * (1.0 - colourPanel.colorSat))
                        }
                        colourPanel.updateColorFromWheel()
                        colorWheelCanvas.requestPaint()
                    }
                }
            }

            // Quick Color Swatches Vertical 2-Column Strip
            Grid {
                columns: 2
                Layout.preferredWidth: 26
                spacing: 2
                Repeater {
                    model: [
                        "#FFFFFF", "#FFFFFF",
                        "#E5E7EB", "#D1D5DB",
                        "#9CA3AF", "#6B7280",
                        "#4B5563", "#374151",
                        "#1F2937", "#111827",
                        "#F59E0B", "#D97706",
                        "#EF4444", "#DC2626",
                        "#EC4899", "#E120A5",
                        "#06B6D4", "#0284C7",
                        "#10B981", "#059669"
                    ]
                    delegate: Rectangle {
                        width: 11
                        height: 8
                        radius: 1
                        color: modelData
                        border.color: "#38383b"
                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                colourPanel.activeHex = modelData
                                if (bridge) {
                                    if (colourPanel.editingStroke) bridge.setStrokeColor(modelData)
                                    else bridge.setFillColor(modelData)
                                }
                                colorWheelCanvas.requestPaint()
                            }
                        }
                    }
                }
            }
        }

        // Readouts: H: 310 S: 75 L: 50 # E120A5
        RowLayout {
            Layout.fillWidth: true
            spacing: 4

            Text { text: "H: " + Math.round(colourPanel.colorHue); color: colourPanel.textSecondary; font.pixelSize: 10 }
            Text { text: "S: " + Math.round(colourPanel.colorSat * 100); color: colourPanel.textSecondary; font.pixelSize: 10 }
            Text { text: "L: 50"; color: colourPanel.textSecondary; font.pixelSize: 10 }
            Item { Layout.fillWidth: true }
            Text { text: "#"; color: colourPanel.textDim; font.pixelSize: 10 }
            TextField {
                Layout.preferredWidth: 58
                Layout.preferredHeight: 18
                font.pixelSize: 10
                text: colourPanel.activeHex.replace("#", "")
                color: "#ffffff"
                background: Rectangle { color: colourPanel.bgField; border.color: colourPanel.borderSubtle; radius: 2 }
                onEditingFinished: {
                    var clean = text.replace("#", "")
                    colourPanel.activeHex = "#" + clean
                    if (bridge) {
                        if (colourPanel.editingStroke) bridge.setStrokeColor("#" + clean)
                        else bridge.setFillColor("#" + clean)
                    }
                    colorWheelCanvas.requestPaint()
                }
            }
        }

        // Opacity Row with sleek PetuniaSlider
        RowLayout {
            Layout.fillWidth: true
            spacing: 6
            Text { text: "Opacidade"; color: colourPanel.textSecondary; font.pixelSize: 10 }
            PetuniaSlider {
                id: opacitySlider
                Layout.fillWidth: true
                from: 0
                to: 100
                value: bridge ? Math.round(bridge.selectedOpacity * 100) : 100
                onMoved: if (bridge) bridge.setOpacity(value / 100)
            }
            Text {
                text: Math.round(opacitySlider.value) + " %"
                color: colourPanel.textPrimary
                font.pixelSize: 10
                Layout.preferredWidth: 32
            }
        }

        Item { Layout.fillHeight: true }
    }
}
