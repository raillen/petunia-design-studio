import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components"

Rectangle {
    id: colourStudio
    Layout.fillWidth: true
    Layout.preferredHeight: 235

    property color bgPanelHeader: "#202021"
    property color bgPanelTabActive: "#2b2b2d"
    property color bgPanelTabInactive: "#1f1f20"
    property color bgPanelBody: "#28282a"
    property color bgField: "#181819"
    property color borderDark: "#181819"
    property color borderSubtle: "#333335"
    property color textPrimary: "#f2f2f5"
    property color textSecondary: "#a1a1a6"
    property color textDim: "#6c6c70"

    property int activeTab: 0 // 0: Colour, 1: Swatches, 2: Stroke, 3: Appearance
    property real colorHue: 310.0
    property real colorSat: 0.75
    property real colorVal: 0.50
    property string activeHex: bridge.currentFill
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
        if (editingStroke) {
            bridge.setStrokeColor(hex)
        } else {
            bridge.setFillColor(hex)
        }
    }

    Connections {
        target: bridge
        function onSelectionChanged() {
            colourStudio.activeHex = bridge.currentFill
            colorWheelCanvas.requestPaint()
        }
    }

    color: bgPanelBody

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // Tab Bar
        Rectangle {
            Layout.fillWidth: true
            height: 25
            color: colourStudio.bgPanelHeader

            RowLayout {
                anchors.fill: parent
                spacing: 0
                Repeater {
                    model: ["Colour", "Swatches", "Stroke", "Appearance"]
                    delegate: Rectangle {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        color: colourStudio.activeTab === index ? colourStudio.bgPanelTabActive : colourStudio.bgPanelTabInactive
                        Text {
                            anchors.centerIn: parent
                            text: modelData
                            color: colourStudio.activeTab === index ? "#ffffff" : colourStudio.textSecondary
                            font.pixelSize: 10
                            font.bold: colourStudio.activeTab === index
                        }
                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: colourStudio.activeTab = index
                        }
                    }
                }
            }
        }

        // Tab 0: Colour Wheel (Exact HSV Donut & Triangle)
        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: colourStudio.activeTab === 0

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 6
                spacing: 4

                // Top row: Color indicators, eyedropper, lock
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 6

                    Item {
                        width: 32
                        height: 20
                        Rectangle {
                            x: 10; y: 2; width: 14; height: 14; radius: 7
                            color: "transparent"
                            border.color: bridge.currentStroke; border.width: 2
                        }
                        Rectangle {
                            x: 2; y: 0; width: 16; height: 16; radius: 8
                            color: bridge.currentFill
                            border.color: "#ffffff"; border.width: 1
                        }
                    }

                    Text {
                        text: Phosphor.eyedropper
                        font.family: "Phosphor"
                        font.pixelSize: 12
                        color: colourStudio.textSecondary
                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: bridge.setTool("eyedropper")
                        }
                    }

                    Item { Layout.fillWidth: true }

                    Text {
                        text: Phosphor.lock
                        font.family: "Phosphor"
                        font.pixelSize: 11
                        color: colourStudio.textDim
                    }
                }

                // Center Row: Color Wheel + Quick Swatches Column
                RowLayout {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 124
                    spacing: 8

                    // Interactive HSV Color Wheel Canvas
                    Canvas {
                        id: colorWheelCanvas
                        Layout.preferredWidth: 124
                        Layout.preferredHeight: 124
                        Layout.alignment: Qt.AlignHCenter

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
                            var triA = (colourStudio.colorHue - 90) * Math.PI / 180
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
                            grad.addColorStop(1, "hsl(" + colourStudio.colorHue + ", 100%, 50%)")
                            ctx.fillStyle = grad
                            ctx.fill()
                            ctx.strokeStyle = "#404044"
                            ctx.lineWidth = 1
                            ctx.stroke()
                            ctx.restore()

                            // Draw Hue ring marker
                            var hRad = colourStudio.colorHue * Math.PI / 180
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
                            var pickX = cx + (p1[0] - cx) * colourStudio.colorSat * 0.7
                            var pickY = cy + (p1[1] - cy) * colourStudio.colorSat * 0.7
                            ctx.beginPath()
                            ctx.arc(pickX, pickY, 4, 0, Math.PI * 2)
                            ctx.fillStyle = colourStudio.activeHex
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
                                    colourStudio.colorHue = Math.round(angle)
                                } else {
                                    colourStudio.colorSat = Math.max(0.1, Math.min(1.0, dist / rIn))
                                    colourStudio.colorVal = 0.5 + (0.5 * (1.0 - colourStudio.colorSat))
                                }
                                colourStudio.updateColorFromWheel()
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
                                        colourStudio.activeHex = modelData
                                        if (colourStudio.editingStroke) bridge.setStrokeColor(modelData)
                                        else bridge.setFillColor(modelData)
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

                    Text { text: "H: " + Math.round(colourStudio.colorHue); color: colourStudio.textSecondary; font.pixelSize: 10 }
                    Text { text: "S: " + Math.round(colourStudio.colorSat * 100); color: colourStudio.textSecondary; font.pixelSize: 10 }
                    Text { text: "L: 50"; color: colourStudio.textSecondary; font.pixelSize: 10 }
                    Item { Layout.fillWidth: true }
                    Text { text: "#"; color: colourStudio.textDim; font.pixelSize: 10 }
                    TextField {
                        Layout.preferredWidth: 56
                        Layout.preferredHeight: 18
                        font.pixelSize: 10
                        text: colourStudio.activeHex.replace("#", "")
                        color: "#ffffff"
                        background: Rectangle { color: colourStudio.bgField; border.color: colourStudio.borderSubtle }
                        onEditingFinished: {
                            var clean = text.replace("#", "")
                            colourStudio.activeHex = "#" + clean
                            if (colourStudio.editingStroke) bridge.setStrokeColor("#" + clean)
                            else bridge.setFillColor("#" + clean)
                            colorWheelCanvas.requestPaint()
                        }
                    }
                }

                // Opacity Row
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 6
                    Text { text: "Opacity"; color: colourStudio.textSecondary; font.pixelSize: 10 }
                    Slider {
                        Layout.fillWidth: true
                        from: 0; to: 100; value: 100
                        onMoved: bridge.setOpacity(value / 100)
                    }
                    Text { text: "100 %"; color: colourStudio.textPrimary; font.pixelSize: 10 }
                }
            }
        }

        // Tab 1: Swatches (Document Swatches)
        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: colourStudio.activeTab === 1

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 8
                spacing: 6

                RowLayout {
                    Layout.fillWidth: true
                    Text {
                        text: "Amostras (" + bridge.swatches.length + ")"
                        color: colourStudio.textPrimary
                        font.pixelSize: 11
                        font.bold: true
                    }
                    Item { Layout.fillWidth: true }
                    Rectangle {
                        width: 76
                        height: 20
                        radius: 3
                        color: addSwatchMouse.containsMouse ? "#3f3f46" : "#27272a"
                        border.color: "#3f3f46"
                        RowLayout {
                            anchors.centerIn: parent
                            spacing: 3
                            Text { text: Phosphor.plus; font.family: "Phosphor"; font.pixelSize: 9; color: "#ffffff" }
                            Text { text: "Adicionar"; font.pixelSize: 9; color: "#ffffff" }
                        }
                        MouseArea {
                            id: addSwatchMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: bridge.addSwatch(colourStudio.activeHex)
                        }
                    }
                }

                ScrollView {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true

                    Flow {
                        width: parent.width
                        spacing: 4

                        Repeater {
                            model: bridge.swatches
                            delegate: Rectangle {
                                width: 22
                                height: 22
                                radius: 3
                                color: modelData
                                border.color: (colourStudio.activeHex.toUpperCase() === modelData.toUpperCase()) ? "#38bdf8" : "#3f3f46"
                                border.width: (colourStudio.activeHex.toUpperCase() === modelData.toUpperCase()) ? 2 : 1

                                MouseArea {
                                    id: swatchMa
                                    anchors.fill: parent
                                    hoverEnabled: true
                                    acceptedButtons: Qt.LeftButton | Qt.RightButton
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: function(mouse) {
                                        if (mouse.button === Qt.RightButton) {
                                            bridge.removeSwatch(modelData)
                                        } else {
                                            colourStudio.activeHex = modelData
                                            if (colourStudio.editingStroke) bridge.setStrokeColor(modelData)
                                            else bridge.setFillColor(modelData)
                                        }
                                    }
                                }

                                ToolTip.visible: swatchMa.containsMouse
                                ToolTip.text: modelData + " (Botão direito para remover)"
                            }
                        }
                    }
                }
            }
        }

        // Tab 2: Stroke
        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: colourStudio.activeTab === 2

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 8
                spacing: 8

                // Espessura
                RowLayout {
                    Layout.fillWidth: true
                    Text { text: "Espessura:"; color: colourStudio.textSecondary; font.pixelSize: 10; Layout.preferredWidth: 64 }
                    Slider {
                        Layout.fillWidth: true
                        from: 0; to: 32; value: bridge.currentStrokeWidth
                        onMoved: bridge.setStrokeWidth(value)
                    }
                    Text { text: Math.round(bridge.currentStrokeWidth) + " pt"; color: colourStudio.textPrimary; font.pixelSize: 10; Layout.preferredWidth: 32 }
                }

                // Extremidade (Cap)
                RowLayout {
                    Layout.fillWidth: true
                    Text { text: "Extremidade:"; color: colourStudio.textSecondary; font.pixelSize: 10; Layout.preferredWidth: 64 }
                    Repeater {
                        model: [
                            { id: "round", label: "Redonda" },
                            { id: "butt", label: "Reta" },
                            { id: "square", label: "Quadrada" }
                        ]
                        delegate: Rectangle {
                            Layout.fillWidth: true
                            height: 20
                            radius: 3
                            color: bridge.selectedStrokeCap === modelData.id ? "#1976d2" : (capMouse.containsMouse ? "#333336" : "#242426")
                            border.color: bridge.selectedStrokeCap === modelData.id ? "#38bdf8" : "#38383b"
                            Text {
                                anchors.centerIn: parent
                                text: modelData.label
                                font.pixelSize: 9
                                color: "#ffffff"
                            }
                            MouseArea {
                                id: capMouse
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: bridge.setStrokeCap(modelData.id)
                            }
                        }
                    }
                }

                // Junção (Join)
                RowLayout {
                    Layout.fillWidth: true
                    Text { text: "Junção:"; color: colourStudio.textSecondary; font.pixelSize: 10; Layout.preferredWidth: 64 }
                    Repeater {
                        model: [
                            { id: "round", label: "Redonda" },
                            { id: "miter", label: "Ângulo" },
                            { id: "bevel", label: "Chanfro" }
                        ]
                        delegate: Rectangle {
                            Layout.fillWidth: true
                            height: 20
                            radius: 3
                            color: bridge.selectedStrokeJoin === modelData.id ? "#1976d2" : (joinMouse.containsMouse ? "#333336" : "#242426")
                            border.color: bridge.selectedStrokeJoin === modelData.id ? "#38bdf8" : "#38383b"
                            Text {
                                anchors.centerIn: parent
                                text: modelData.label
                                font.pixelSize: 9
                                color: "#ffffff"
                            }
                            MouseArea {
                                id: joinMouse
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: bridge.setStrokeJoin(modelData.id)
                            }
                        }
                    }
                }

                // Estilo / Traço
                RowLayout {
                    Layout.fillWidth: true
                    Text { text: "Padrão:"; color: colourStudio.textSecondary; font.pixelSize: 10; Layout.preferredWidth: 64 }
                    Repeater {
                        model: [
                            { id: "", label: "Contínuo" },
                            { id: "6, 4", label: "Tracejado" },
                            { id: "2, 3", label: "Pontilhado" }
                        ]
                        delegate: Rectangle {
                            Layout.fillWidth: true
                            height: 20
                            radius: 3
                            color: dashMouse.containsMouse ? "#333336" : "#242426"
                            border.color: "#38383b"
                            Text {
                                anchors.centerIn: parent
                                text: modelData.label
                                font.pixelSize: 9
                                color: "#ffffff"
                            }
                            MouseArea {
                                id: dashMouse
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: bridge.setStrokeDash(modelData.id)
                            }
                        }
                    }
                }

                Item { Layout.fillHeight: true }
            }
        }

        // Tab 3: Appearance
        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: colourStudio.activeTab === 3

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 8
                spacing: 6

                Text { text: "Aparência da Seleção"; color: colourStudio.textPrimary; font.pixelSize: 11; font.bold: true }

                Rectangle {
                    Layout.fillWidth: true
                    height: 26
                    radius: 3
                    color: "#202022"
                    border.color: "#333335"
                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: 4
                        spacing: 6
                        Rectangle { width: 14; height: 14; radius: 2; color: bridge.currentFill; border.color: "#555" }
                        Text { text: "Preenchimento: " + bridge.currentFill; color: colourStudio.textPrimary; font.pixelSize: 10; Layout.fillWidth: true }
                    }
                }

                Rectangle {
                    Layout.fillWidth: true
                    height: 26
                    radius: 3
                    color: "#202022"
                    border.color: "#333335"
                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: 4
                        spacing: 6
                        Rectangle { width: 14; height: 14; radius: 2; color: "transparent"; border.color: bridge.currentStroke; border.width: 2 }
                        Text { text: "Traço: " + bridge.currentStroke + " (" + Math.round(bridge.currentStrokeWidth) + " pt)"; color: colourStudio.textPrimary; font.pixelSize: 10; Layout.fillWidth: true }
                    }
                }

                Rectangle {
                    Layout.fillWidth: true
                    height: 26
                    radius: 3
                    color: "#202022"
                    border.color: "#333335"
                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: 4
                        spacing: 6
                        Text { text: "Opacidade: " + Math.round(bridge.selectedOpacity * 100) + "%  |  Modo: " + (bridge.selectedBlendMode || "normal"); color: colourStudio.textSecondary; font.pixelSize: 10; Layout.fillWidth: true }
                    }
                }

                Item { Layout.fillHeight: true }
            }
        }
    }
}
